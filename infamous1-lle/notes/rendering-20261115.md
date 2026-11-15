# Renderização — 2026-11-15

## Antisserrilhamento implementado

Novo modo `PS3_AA=smaa-fxaa`, exposto no launcher e no menu F1:

- SMAA na resolução interna antes de ampliar/reduzir a imagem.
- FXAA depois da escala, incluindo as bordas introduzidas pela reconstrução.
- Limiares do FXAA combinado: relativo 0,0833, mínimo 0,0312. O modo FXAA
  existente mantém 0,166 / 0,0833.
- Perfil Qualidade: 3840×2160 interno, anisotrópico 16×, SMAA + FXAA,
  MetalFX quando é necessário ampliar, nitidez adicional zero. Quando a
  saída é menor, continua sendo usado o Lanczos já existente.

O perfil aumenta o custo de GPU e pode suavizar texto e texturas. Não há
acumulação temporal: cintilação em movimento e detalhes menores que um pixel
continuam sendo limitações. Não foi comprovado serrilhado zero em toda a campanha.

A configuração local recebeu esse perfil. Backup:
`~/Library/Application Support/infamous1-recomp/launcher.json.bak-before-hybrid-aa-20261115`.
É necessário reabrir o launcher e o jogo para carregar os executáveis novos.

Validação:

- `npm run build`: passou; bundle Tauri macOS reconstruído.
- `cargo test env_graphics_options --lib`: passou, incluindo o modo novo.
- `cmake --build infamous1-lle/build --target Infamous1Recomp -j 4`: passou.
- Teste real Metal `test_metal_hybrid_aa.m`: 496 pixels adicionais filtrados
  sobre SMAA na diagonal sintética; 229 pixels com cobertura parcial;
  região uniforme e alpha preservados. Reproduzir com
  `python3 infamous1-lle/tools/test_hybrid_aa.py` fora de um sandbox sem GPU.
- `test_metal_downscale.py`: passou. Checkerboard de um pixel reduzido
  de 192×108 para 64×36 saiu cinza uniforme (127); branco permaneceu 255.
- Captura real processada pelos shaders novos:
  `out/aa-review-20261115/hybrid.png`. A entrada está em `before.ppm`.
  Essa comparação é estática, não mede cintilação temporal.
- Executável novo com resolução 3× e AA combinado rodou 240 s, gerou
  capturas 3840×2160 após o passe final em `out/aa-quality-20261115/`, sem
  falhas de pipeline ou command buffer registradas. O roteiro abriu o menu
  de poderes; não constitui benchmark de gameplay nem valida toda a campanha.
- Patch completo do toolkit reaplicado numa base limpa; fontes do renderer
  e testes conferidos contra os arquivos locais.

## Sombra e outros defeitos

O log de `out/shadow-probe-20261115.log` confirmou mapas 1024×1024 em
zeta `0x0F340000`, comparação e PCF ativos nos programas
`395b63487dde8cc0`, `6be19b39da03d1b9` e `37ba2ce71fec0dda`.
A sombra fragmentada da imagem do usuário ainda não tem correção comprovada.

**Resolvido depois (mesmo dia):** não é erro de tradução. Cenário: o save
copiado, o roteiro de controle do probe, o Cole andando e a câmera de cima
no terraço (`out/shadow-ab1..3`, `out/shadow-soft-*`). As etapas da sombra
foram alternadas ao vivo na mesma cena (`PS3_TOGGLE_FILE`):
`ZTEX_CROP=0` e `SHADOW_CMP=0` estragam todas as sombras (então o recorte
e a ordem atuais estão certos); `DEPTH_BOUNDS`, `SHADE_BACKFACE`,
`POLY_OFFSET`, `DEPTH_CLAMP` e `SHADOW_D24` não mudam a sombra do Cole.
Em 1× e 3× a mesma cena mostra o mapa ganhando resolução com a escala (a
sombra do arame farpado só aparece nítida em 3×). Os degraus são o próprio
mapa de 1024×1024 do jogo, filtrado em 2×2 texels como no RSX: com o sol
baixo cada texel fica esticado no chão.

Correção: `PS3_SHADOW_FILTER` (auto) compara 4×4 texels com pesos
(1-f, 1, 1, f)/3 por eixo quando a resolução interna passa de 720p; em 1×
o shader é o mesmo de antes, byte a byte. `libs/video/tests/test_shadow_soft.c`:
3220 verificações (descritor, texto do shader, rampa de 3 texels sem saltos).
No jogo os três FPs acima saem com `soft 1`, sem erro de shader.
Antes/depois em 1.5× com o mesmo binário (`PS3_SHADOW_FILTER=0` reproduz o
antigo): a sombra do Cole no chão aberto perde os degraus, e no barril as
bordas ficam suaves; o "depois" rodou a ~34–35 fps headless. Efeito
colateral: sombras muito finas (arame farpado) ficam mais fracas.

Corrigido um defeito independente: `FRONT_AND_BACK` era traduzido como
`FRONT`, deixando as faces traseiras serem desenhadas. O engine agora não
submete polígonos nesse estado; linhas e pontos continuam sendo desenhados.
`test_cull_both.c`: seis verificações passaram. Não foi identificado esse
estado como a causa da sombra da imagem.

O conjunto antigo `test_rsx_draw_engine.c` apresentou quatro falhas antes
da correção: três expectativas sobre realocação de superfícies e uma sobre
indexação de triângulos. Não foi declarado que a suíte inteira passou.

`run.sh` agora exige os dois arquivos install1/infamous1.psarc_s e
install2/infamous2.psarc_s antes do boot, evitando o loop de carregamento
causado pela ausência do segundo arquivo. Sintaxe e caminho de falha verificados.

## Pop-in: investigação, sem alteração de distância de desenho

Ainda não foram localizadas as constantes de distância de streaming/LOD;
nenhuma distância foi aumentada nesta mudança.

O patch local do RPCS3 "Disable Mesh Trimming", por Yahfz, diz corrigir
renderização acima da resolução do PS3. Para este EBOOT ele escreve
`0x35000000` em `0x006158A0`. O endereço pertence ao job SPU
`jobmod_2e6aac472eee_ec2c0`, imagem 202, fonte `0x00611380`, LS `0x08520`;
a instrução original é `0x1C450191`. A substituição é um retorno SPU.
Existe uma função recompilada em `0x08520`, com chamadas diretas geradas.
Assim, apenas escrever o opcode na RAM, ou trocar a função no registro de
dispatch, não é suficiente para garantir que todas as chamadas recompiladas
executem a alteração. O patch não foi habilitado nem anunciado como solução
de pop-in. A próxima investigação deve separar cortes indevidos de malha,
troca de LOD e atraso real de carregamento de recursos.
