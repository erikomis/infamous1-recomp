# Roadmap

🌐 [English](ROADMAP.md) · **Português (BR)**

Estado do port nativo do inFAMOUS 1 (`NPUA80480`) para macOS/arm64 e o que vem
a seguir. `[x]` = feito e verificado; `[ ]` = a fazer. Evidências e logs estão em
[NOTES.md](NOTES.md); os recursos trazidos do port do inFAMOUS 2 estão em
[PORT_FEATURES.md](PORT_FEATURES.md).

## Estado em resumo (2026-11-11)

- **Roda:** boot, logos dos estúdios, vídeos Bink, a tela inicial 3D, START, a explosão de abertura e o Cole no tutorial do prólogo ("Use the right stick to look up at the chopper"). Jogado headless com um controle por script (`tools/play_prologue.sh`); o launcher também inicia o jogo numa janela Metal.
- **Velocidade:** ~44 fps no menu e nas cutscenes, headless, em Apple silicon (o console mirava 30). O fps em gameplay ainda não foi medido.
- **Rebuild:** o `infamous1-lle/tools/rebuild.py` compila o jogo a partir da cópia do jogador num comando; numa pasta limpa ele reproduziu o build testado byte a byte e bootou até o tutorial.
- **App:** o bundle macOS compila, 72 testes passam (3 opcionais ignorados) e o botão Jogar inicia o jogo (`BootCheck: type=2, titleId=NPUA80480`). A tela de instalação/rebuild foi religada (ainda não testada clicando); a atualização automática está desligada.
- **Release:** nenhum ainda.
- **O que falta para um "1.0 jogável no Mac":** jogar a história inteira (ninguém passou do tutorial) e corrigir o que aparecer, levar os jobs de iluminação e SSAO para a GPU por velocidade, e um release do app.
- **Ainda não:** Windows e Linux.

## Histórico

*Guardado como registro de como o port chegou aqui.*

- **2026-11-06/08, bring-up:** o boot LLE (firmware, SPURS, policy module `Wws_Job`) no fork do inFAMOUS 1, seguindo o modelo do port do inFAMOUS 2. O boot travava no `boot4`/`boot5`, com `sys_memory allocate(0x100000)` retornando para sempre (26.865 tentativas, `ENOMEM` depois de 125 alocações até `0x4FF00000`).
- **2026-11-09, a tela inicial:** a correção da janela de memória (`PS3_MEM_ALLOC_SIZE`) e o boot por HDD (`CATEGORY` do `cellGame`) levaram o recomp à tela inicial 3D "Press the Start Button", e depois da explosão até o Cole no tutorial.
- **2026-11-10, o app e a comparação de recursos:** o launcher do inFAMOUS 2 adaptado (título, caminhos, ids e configurações próprios), anisotrópico 16× verificado, o candidato a job de luz achado, e este roadmap.

---

## O que falta para ser 100% jogável

### Repositório (ganhos rápidos, 2026-11-11)

- [x] O patch do toolkit neste repositório: `tools/setup_toolkit.sh` e `rebuild.py` procuravam `ps3recomp-fork-770660a-with-upstream-prs.patch`, que só existia na árvore do inFAMOUS 2, então um checkout limpo não compilava. Agora está em `infamous1-lle/patches/`, gerado a partir do checkout do toolkit contra o qual o port compila: o patch do inFAMOUS 2 mais a correção `PS3_MEM_ALLOC_SIZE`. Verificado: o `setup_toolkit.sh` aplica num clone limpo do `770660a`.
- [x] Screenshots do port em `docs/screenshots/` e uma galeria no topo do README, como nos repositórios do inFAMOUS 2 e do Resistance (frames headless do `out/start-run`).
- [x] CI do launcher (macOS, Windows, Linux) e das ferramentas Python/shell (`.github/workflows/`), mais o Dependabot para os pins das actions. Cada passo rodou localmente antes: os self-tests das ferramentas, o staging, `rebuild.py --check-json`, `npm run typecheck/build`, `cargo test --locked` (72 passaram).
- [x] O mock da interface do launcher (`launcher/scripts/ui-mock.js`) mostrava nomes, caminhos e saves do inFAMOUS 2.
- [x] O `rebuild.py` não levava o `runtime_overrides/cellGame.c` (a correção do boot por HDD) junto com os fontes do port, então um rebuild perderia a correção; a checagem do staging pegou isso. Faltava também o `infamous1-lle/README.md`, que o rebuild copia.

### Bloqueadores

- [x] Boot além da janela de memória (o alocador terminava 256 MB depois da base; agora `PS3_MEM_ALLOC_SIZE`, 768 MB no `run.sh`). Evidência: `out/boot/boot7.log`, zero `null-write`, 66 chamadas de alocação no total, o fluxo inteiro até os vídeos Bink e a "Game State Thread".
- [x] Tela inicial → gameplay: START (`0x0008`, não ✕) sai da tela inicial; a explosão e o tutorial renderizam (`out/start-run/`).
- [ ] **Jogar a história inteira.** Ninguém passou do tutorial. O `tools/play_prologue.sh` grava os primeiros minutos; as missões seguintes pedem uma sessão em janela com controle.
- [ ] Estabilidade em sessões longas: memória, streaming, travamentos e engasgos ao longo de horas.
- [ ] Salvar e carregar na campanha inteira, verificado em janela (o gerenciador reconhece `BCUS98119_INFAMOUS_*`; nenhum save original foi alterado).

### Gráficos

- [x] Filtragem anisotrópica até 16× (`out/aniso-smoke.log`, `out/aniso16/`).
- [ ] Capturas A/B das alavancas de imagem (FXAA/SMAA, `PS3_RES_SCALE`, MetalFX, nitidez) na mesma cena, com o `tools/capture.sh`.
- [ ] Verificar a renderização em cenas mais adiante (iluminação, sombras, água, partículas) em janela; as correções do inFAMOUS 2 foram achadas assim.

### Estabilidade

- [ ] Rodar o `tools/profile_run.sh` em gameplay: fps por período e código mais quente, como linha de base antes do trabalho de iluminação na GPU.
- [ ] Um aquecimento do cache de shaders, para os engasgos de primeiro uso acontecerem uma vez só.

### Saves

- [x] Gerenciador de saves: listar, backup, restaurar, zip e troca com o RPCS3, testado em pastas temporárias.
- [ ] Verificar com o save de uma sessão real, em janela.

### Controles

- [x] Controle, teclado e mouse mapeados pelo runtime compartilhado, bindings testados.
- [ ] Todas as combinações de dispositivos em janela (DualSense, DualShock 4, Xbox, teclado e mouse).
- [ ] O menu no jogo (F1 / Cmd+, / L3+R3) neste jogo.

### Rebuild

- [x] Portar o `infamous1-lle/tools/rebuild.py` para este jogo (2026-11-11). Ele era a receita do inFAMOUS 2. Agora: `--code-end 0x55EB44`; o policy module `Wws_Job` recortado do endereço `0x69E400` (11.648 bytes, achado pelos program headers; o do inFAMOUS 2 ficava em `0x8E5B00`); a image de áudio com relift dos retornos de chamada (`relift_spu_returns.py`); o `jobmod_542edebc6499_e80` com relift em `0x18BB0`; sem módulos de dados nem relocáveis; o EBOOT testado é APP_VER 01.00, SHA-256 `5e22402a…`.
- [x] Um `rebuild.py` de ponta a ponta numa pasta limpa: todos os passos passaram (DLC pulado, não existe), `lle/`, `lle_spu/`, `recompiled/` e o `spu/` inteiro saíram idênticos byte a byte ao build testado, e o binário novo foi dos logos, pela tela inicial e a explosão, até o tutorial numa execução headless de 200 s, sem abort de SPU.
- [ ] A tela de instalação/rebuild do app: religada (ela roda o `rebuild.py`), ainda não testada clicando.
- [ ] O passo do toolkit clonando pela rede dentro do `rebuild.py` (a execução acima usou `--toolkit` num clone limpo que o `setup_toolkit.sh` tinha preparado).
- [ ] Procurar os relocatable job lifts do inFAMOUS 2 neste EBOOT em outros formatos: o scanner achou zero blobs com a assinatura esperada (isso não exclui outros formatos).

---

## Depois de jogável (melhorias)

### Velocidade: iluminação e SSAO na GPU

No inFAMOUS 2 isso levou a caminhada de New Marais de 12 para 15 fps e de 21 para 28 fps. Aqui exige o trabalho específico deste jogo, porque os hooks do segundo jogo dependem de outros jobs, layouts de local store, buffers e sincronização.

1. [ ] Identificar em um trace de gameplay os jobs que produzem iluminação e SSAO.
2. [ ] Mapear os parâmetros de cada job, os buffers de entrada/saída e os sinais de término.
3. [ ] Confirmar o job de luz. O candidato líder é a image **209** = `jobmod_6cbbde2f1bfc_e9d68` (entry `0xDD68`, 43.616 B, fonte streamada de `0x61E680` para LS `0x4000`), o overlay mais streamado no boot e nas cutscenes (13×), o mesmo perfil do job de luz do inFAMOUS 2. Alternativas: image 215 (`jobmod_cbe85abb417c_e12058`, 6×) e 218 (`jobmod_d46637430ef9_e5bb8`, 6×). O teste: stubar a rotina por tile do candidato com `spu_replace_function` e olhar o efeito visual; no inFAMOUS 2, um job de luz quebrado deixava props brilhantes estourados em branco.
4. [ ] Comparar as fórmulas com os shaders Metal do inFAMOUS 2 e adaptar o que divergir.
5. [ ] Comparar SPU contra GPU na mesma cena antes de ligar os atalhos.
6. [ ] Portar `gpu_light.c` e `gpu_ssao.c` (placeholders por enquanto: os do inFAMOUS 2 referenciam os símbolos `jobmod_*` daquele jogo).

### Ultrawide e o HUD

- [ ] Mapear as câmeras e o shader de composição final. O patch de câmera existente é limitado ao hash do EBOOT do inFAMOUS 2, por isso a opção está oculta aqui.

### O app

- [ ] Assistente de instalação/rebuild (veja Rebuild).
- [ ] Atualização automática com endpoint e chave próprios; os do inFAMOUS 2 não estão ativos.
- [ ] Um `.dmg` de release assinado.

### Outras plataformas

- [ ] Windows e Linux.

### Fora do escopo por enquanto

- Missões secundárias repetitivas: design de jogo, não defeito técnico.

---

## Problemas do jogo original

Levantados pelo Erik (2026-11-10), com o plano técnico de cada item. Os problemas de imagem e desempenho o port ataca por natureza (alavancas do runtime); os de lógica de jogo exigem patches cirúrgicos no lift (`game_patches.cpp` / runtime overrides), cada um com engenharia reversa própria.

### Prioridade alta

| Problema | O que acontecia no original | Plano no port | Estado |
|---|---|---|---|
| Quedas de FPS | Oscilações durante explosões, combates e áreas movimentadas | 1) O recomp já roda ~44 fps (o console mirava 30). 2) Job de luz/SSAO movido para a GPU (acima). 3) O cache de shaders evita engasgos de tradução | 🟡 parcial (2 e 3 pendentes) |
| Aliasing | Bordas serrilhadas em prédios, fios e objetos | `PS3_AA=fxaa/smaa` + `PS3_UPSCALE=metalfx` + `PS3_RES_SCALE=2` (1440p), já no launcher; validar o A/B visual | 🟡 disponível, validação pendente |
| Escalada | O Cole agarrava objetos indesejados ou não alcançava o ponto certo | Patch de lógica no lift: mapear raycast/agarramento (`game_patches.cpp`); pode precisar de ajuste por constante | ⚪ não iniciado (pesado) |
| Pop-in | Objetos e detalhes surgiam conforme o jogador se aproximava | O streaming já roda de SSD local (mais rápido que o PS3). A distância de desenho é constante do jogo: mudar exige achar as constantes de LOD/streaming no EBOOT | 🟡 parcial (streaming ok, LOD não) |

### Prioridade média

| Problema | O que acontecia | Plano no port | Estado |
|---|---|---|---|
| Câmera | Enquadramentos ruins em espaços apertados e durante escaladas | Mapear as funções de enquadramento/colisão da câmera no lift e ajustar; o `PS3_ASPECT=21:9` é separado e também pendente | ⚪ não iniciado |
| Combate | Inimigos com precisão elevada atacando de longe | Patch de lógica: achar as constantes de precisão/alcance no código do jogo | ⚪ não iniciado |
| IA dos inimigos | Comportamentos repetitivos, reações pouco naturais | Idem: cada comportamento é um ponto de mudança; custo alto, ganho subjetivo | ⚪ não iniciado |
| Checkpoints | Trechos que obrigavam a repetir partes frustrantes | Achar os pontos de respawn/save no código do jogo e afrouxar o espaçamento | ⚪ não iniciado |
| Texturas | Superfícies com baixa definição e nitidez | **Validado 2026-11-10**: aniso 16×; mais `PS3_RES_SCALE`, `PS3_SHARPEN` e MetalFX no launcher; mods de textura (pacotes PNG) também disponíveis | 🟢 validado |

### Ordem de ataque

1. **Validar os presets de imagem** (AA + `RES_SCALE` 1440p): A/B de captura, entrega imediata, zero risco (sem tocar no código do jogo).
2. **`profile_run.sh`**: fps por período e código mais quente em gameplay, linha de base antes da iluminação na GPU.
3. **Iluminação/SSAO na GPU**: confirmar a image 209 por A/B, mapear o layout/labels de LS, portar o `gpu_light.c`.
4. **Pop-in/LOD**: caçar as constantes de streaming no EBOOT.
5. **Câmera / escalada / combate / IA / checkpoints**: cada um um projeto de engenharia reversa separado, por ordem de incômodo.

### Notas técnicas

- Patches de lógica seguem o padrão do `game_patches.cpp`: filtrados pelo hash do program header, nunca aplicar um patch do inFAMOUS 2 neste EBOOT.
- Toda validação de imagem é A/B de captura (`tools/capture.sh`) na mesma cena, com a máquina livre (o `tools/profile_run.sh` se serializa sozinho).
- Antes de mexer na iluminação na GPU, fotografar a linha de base com o job SPU original.

---

## Feito e concluído

### Boot e pipeline

- [x] Boot LLE com o firmware (liblv2, libsysmodule, libsre) no fork do inFAMOUS 1.
- [x] Extrator dos módulos de job embutidos no EBOOT (`extract_eboot_jobmods.py`), o lift deles (`lift_spu_jobs.sh`, `lift_datamods.py`, `lift_relocmods.py`) e os retornos SPU (`relift_spu_returns.py`).
- [x] Boot por HDD: o `cellGame` lê o `CATEGORY` do `PARAM.SFO`; o `run.sh` passa `PS3_BOOT_PATH`. Teste isolado `tests/test_game_boot.c`.
- [x] A janela de memória (`PS3_MEM_ALLOC_SIZE`).
- [x] A limpeza recursiva achada com o trace do sistema de arquivos (`func_00105550`).

### Do menu ao gameplay

- [x] Logos dos estúdios, vídeos Bink (`splash_america.bik`, `tv_default_feed_2x2.bik`), música e streaming.
- [x] Tela inicial 3D, START, a explosão, o Cole no tutorial.
- [x] `tools/play_prologue.sh` reancorado nas evidências do `out/start-run` (o mundo começa em 145-200 s).

### Gráficos e desempenho

- [x] Janela Metal em 1280×720, 32 pipelines de shader compilados no boot.
- [x] Filtragem anisotrópica até 16×.
- [x] ~44 fps no menu e nas cutscenes, headless.

### App e ferramentas

- [x] Launcher Tauri adaptado (título, caminhos, ids, configurações, inglês/português), 72 testes passam, bundle gerado.
- [x] Detecção da instalação pelo `PARAM.SFO` (`NPUA80480`); saves `BCUS98119` aceitos, `BCUS98125` rejeitados.
- [x] O `tools/capture.sh` recorre ao venv do toolkit ou ao `python3`.
