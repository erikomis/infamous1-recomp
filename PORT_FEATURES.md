# Recursos do inFAMOUS 2 no port do inFAMOUS 1

Revisão de 2026-11-10. O inFAMOUS 1 continua usando seu próprio EBOOT,
firmware levantado, registro SPU, boot HDD e correções de retornos SPU.

Correções dos problemas do jogo original (FPS, aliasing, texturas, câmera,
escalada etc.) são planejadas em [ROADMAP.md](ROADMAP.md).

| Recurso | Estado neste port | Validação / limite |
|---|---|---|
| Launcher Tauri, PT-BR/inglês | Adaptado e compilado | Iniciou o recomp em janela no M4 Pro |
| Identificação de instalação | Adaptada | PARAM.SFO NPUA80480; executável local detectado |
| Configurações persistentes | Adaptadas | Diretório `infamous1-recomp`, separado do segundo jogo |
| Janela, fullscreen, monitor, vsync | Disponíveis pelo runtime compartilhado e launcher | Mapeamento de opções testado; janela Metal iniciada |
| MetalFX, FXAA/SMAA, escala interna | Disponíveis pelo runtime compartilhado e launcher | Mapeamento testado; não houve comparação visual de cada combinação |
| Filtragem anisotrópica (até 16×) | Disponível e validada no inF1 (`out/aniso-smoke.log`, `out/aniso16/`) | `PS3_ANISO` aplica só em texturas com mipmaps; troca também no menu do jogo |
| Cor, gamma, LUT, nitidez | Disponíveis pelo runtime compartilhado e launcher | Mapeamento testado |
| Cache de shaders | Já herdado; gerenciamento no launcher | Usa o cache específico deste port |
| Mouse, teclado, controle, vibração e áudio | Adaptados no launcher | Testes dos bindings e variáveis; todas as combinações de dispositivos não foram testadas |
| Mods de textura | Interface e runtime disponíveis | Nenhum pacote de terceiros foi instalado |
| Backups, restauração, ZIP e intercâmbio com RPCS3 | Adaptados | Testes em diretórios temporários; reconhece BCUS98119/NPUA80480 e rejeita saves do 2 |
| Troféus locais, logs e relatórios | Interface adaptada | Sem sincronização ou envio externo automático |
| Atalhos / integração Steam | Código adaptado | Nenhuma biblioteca Steam foi alterada; requer ação na interface |
| Iluminação GPU / SSAO GPU | **Pendente de engenharia específica do jogo** | Hooks do 2 dependem de jobs, layout LS, buffers e sincronização diferentes; não habilitados |
| Remoção de passes de oclusão do 2 | **Não aplicável por cópia direta** | Hashes de shaders do 2 removidos do ambiente padrão do launcher |
| Ultrawide com câmera/HUD corrigidos | **Pendente** | O patch de câmera existente é limitado ao hash do EBOOT do 2; opção oculta |
| Relocatable job lifts do 2 | **Não encontrados pelo scanner no EBOOT do 1** | Zero blobs com a assinatura esperada; não significa ausência de outros formatos |
| Instalador/rebuild completo pelo launcher | **Pendente** | Receita herdada do 2 não é executada pelo launcher; build local CMake continua funcionando |
| Atualização automática | **Desativada nesta distribuição local** | Nenhum endpoint/chave de atualização do 2 fica ativo |

Não há ganho de FPS medido decorrente desta adaptação do launcher. Recursos
de imagem podem aumentar o custo da GPU; desempenho de cada preset ainda
precisa de comparação usando a mesma cena.

## Evidências

- `infamous1-lle/out/diagnostics/launcher-tests.log`: 72 testes passaram,
  3 testes opcionais ignorados.
- `infamous1-lle/out/diagnostics/launcher-build.log`: bundle macOS gerado.
- Log de execução pela interface: `~/Library/Application Support/infamous1-recomp/logs/game.log`.
- Saves do primeiro jogo observados no RPCS3: `BCUS98119_INFAMOUS_0`;
  nenhum save do RPCS3 foi modificado durante a adaptação.
- Launcher: `launcher/src-tauri/target/release/bundle/macos/infamous1-recomp.app`.

## Trabalho restante para a parte gráfica específica

1. Identificar no trace de gameplay os jobs que produzem iluminação e SSAO.
2. Mapear os parâmetros de cada job, buffers de entrada/saída e sinais de término.
3. Comparar as fórmulas com os shaders Metal do 2; adaptar o que divergir.
4. Comparar SPU versus GPU na mesma cena antes de habilitar os atalhos de execução.
5. Mapear câmeras e o shader de composição final para ultrawide/HUD.
