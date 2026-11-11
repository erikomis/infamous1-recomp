# Problemas do jogo original — roadmap de correções no port

Lista levantada pelo Erik (2026-11-10), com o plano técnico de cada item para
o port (static recomp). Problemas de **imagem/desempenho** o port ataca por
natureza (alavancas do runtime); problemas de **lógica de jogo** exigem
patches cirúrgicos no lift (`game_patches.cpp` / runtime overrides), cada um
com engenharia de reverse engineering própria.

Estado e evidências das validações ficam em `NOTES.md`; visão geral de
recursos em `PORT_FEATURES.md`.

## Prioridade alta

| Problema | O que acontecia no original | Plano no port | Estado |
|---|---|---|---|
| Quedas de FPS | Oscilações durante explosões, combates e áreas movimentadas | 1) Recomp já roda ~44 fps (console mirava 30). 2) Job de luz/SSAO movido pra GPU (no if2: walk 12→15 e 21→28 fps). Candidato do job de luz já identificado: image 209 = `jobmod_6cbbde2f1bfc_e9d68` (ver NOTES.md). 3) Cache de shaders evita hitches de tradução | 🟡 parcial (2 e 3 pendentes) |
| Aliasing | Bordas serrilhadas em prédios, fios e objetos | `PS3_AA=fxaa/smaa` + `PS3_UPSCALE=metalfx` + `PS3_RES_SCALE=2` (1440p nativo) — já no launcher; validar A/B visual no inF1 | 🟡 disponível, validação pendente |
| Escalada | Cole agarrava objetos indesejados ou não alcançava o ponto certo | Patch de lógica no lift: mapear raycast/agarramento (`game_patches.cpp`); pode precisar de tuning por constante | ⚪ não iniciado (pesado) |
| Pop-in | Objetos/detalhes surgiam conforme o jogador se aproximava | Streaming já roda de SSD local (mais rápido que PS3). Draw distance é constante do jogo; mudar exige achar as constantes de LOD/streaming no EBOOT | 🟡 parcial (streaming ok, LOD não) |

## Prioridade média

| Problema | O que acontecia | Plano no port | Estado |
|---|---|---|---|
| Câmera | Enquadramentos ruins em espaços apertados e durante escaladas | Mapear funções de enquadramento/colisão de câmera no lift e ajustar; nota: `PS3_ASPECT=21:9` (ultrawide hor+) é separado e também pendente de mapear | ⚪ não iniciado |
| Combate | Inimigos com precisão elevada e atacando de longe | Patch de lógica: achar constantes de acurácia/alcance no código do jogo | ⚪ não iniciado |
| IA dos inimigos | Comportamentos repetitivos, reações pouco naturais | Idem — cada comportamento é um ponto de mudança; alto custo, ganho subjetivo | ⚪ não iniciado |
| Checkpoints | Trechos que obrigavam a repetir partes frustrantes | Achar pontos de respawn/save no código do jogo e afrouxar espaçamento | ⚪ não iniciado |
| Texturas | Superfícies com baixa definição e nitidez | **Validado 2026-11-10**: aniso 16× (`out/aniso-smoke.log`, `out/aniso16/`); + `PS3_RES_SCALE`, `PS3_SHARPEN`, MetalFX no launcher; mods de textura (PNG packs) também disponíveis | 🟢 validado |

## Prioridade baixa

| Problema | O que acontecia | Plano no port | Estado |
|---|---|---|---|
| Missões repetitivas | Atividades secundárias repetitivas | Design de jogo, não defeito técnico — fora do escopo do port | ⚪ fora de escopo |

## Ordem recomendada de ataque

1. **Validar presets de imagem** (AA + RES_SCALE 1440p) — A/B de captura,
   entrega imediata, zero risco (sem tocar em código do jogo).
2. **`profile_run.sh`** — fps por período + código mais quente em gameplay,
   baseline antes do GPU light.
3. **GPU light/SSAO** — confirmar image 209 como job de luz via A/B
   (rotina por-tile stubada → efeito visual característico), mapear layout
   LS/labels, portar `gpu_light.c` do if2. Ganho maior de FPS em combate.
4. **Pop-in/LOD** — caçar constantes de streaming no EBOOT.
5. **Câmera / escalada / combate / IA / checkpoints** — cada um é um projeto
   de reverse engineering separado; atacar por ordem de incômodo.

## Notas técnicas

- Patches de lógica seguem o padrão do `game_patches.cpp` (filtragem por
  hash de header de programa — nunca aplicar patch do if2 no EBOOT do inF1).
- Toda validação de imagem deve ser A/B de captura (`tools/capture.sh`) na
  mesma cena, com a máquina livre (`tools/profile_run.sh` se auto-serializa).
- Antes de mexer em GPU light, fotografar o baseline com o job SPU original.
