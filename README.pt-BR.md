# inFAMOUS 1 — recompilação estática para PC (macOS)

🌐 [English](README.md) · **Português (BR)**

Um port nativo do **inFAMOUS** (PS3, versão digital `NPUA80480`) para
**macOS/arm64**, feito por recompilação estática: o código PowerPC e SPU do
jogo é traduzido para C/C++ e compilado num executável nativo, com gráficos em
Metal. Não é um emulador.

**Ele dá boot, mostra os logos e vídeos, chega à tela inicial 3D e passa da
explosão de abertura até o Cole no tutorial** ("Use the right stick to look up
at the chopper"), a ~44 fps em Apple silicon em execuções headless. Ele
recompila o jogo a partir da sua própria cópia ([como](#compilando-a-partir-do-código)).

> [!WARNING]
> **Alpha inicial.** O jogo roda do boot até o tutorial do prólogo, só em macOS
> com Apple silicon. A campanha completa e sessões longas não foram validadas.
> O que falta está em [ROADMAP.pt-BR.md](ROADMAP.pt-BR.md).

> **Este repositório não contém dados do jogo, firmware, chaves da Sony nem
> nada traduzido a partir deles.** Cada pessoa gera tudo isso localmente, a
> partir da sua própria cópia legítima. Projeto de fã, sem vínculo com a Sony
> Interactive Entertainment nem com a Sucker Punch Productions.

Construído sobre o [ps3recomp](https://github.com/sp00nznet/ps3recomp) e,
principalmente, sobre o **fork do inFAMOUS 1** do
[NoahBPeterson](https://github.com/NoahBPeterson/ps3recomp), que trouxe o LLE do
firmware e as correções que fizeram o jogo rodar no Mac; o launcher e o runtime
compartilhado vêm do [port do inFAMOUS 2](https://github.com/erikomis/infamous2-recomp).
Veja [CREDITS.pt-BR.md](CREDITS.pt-BR.md).

## Estado

| | |
| --- | --- |
| Boot, LLE do firmware, SPURS, módulos de job SPU | ✅ |
| Logos dos estúdios, vídeos Bink, telas de carregamento | ✅ |
| Tela inicial 3D ("Press the Start Button") | ✅ |
| Novo jogo → explosão de abertura → Cole no tutorial | ✅ |
| Desempenho | ✅ ~44 fps no menu e nas cutscenes, headless, em Apple silicon (o console mirava 30). O fps em gameplay ainda será medido com `tools/profile_run.sh` |
| Filtragem anisotrópica até 16× | ✅ verificada neste jogo |
| Resolução interna, MetalFX, FXAA/SMAA, nitidez, cor | 🧪 pelo runtime compartilhado e pelo launcher, opções mapeadas; sem A/B visual por combinação ainda |
| Controle, teclado e mouse, vibração | 🧪 adaptados no launcher, bindings testados; nem todas as combinações de dispositivos |
| Menu no jogo (F1 / Cmd+, / L3+R3) | 🧪 runtime compartilhado, ainda não verificado neste jogo |
| Saves e troféus, gerenciador de saves no app | ✅ saves locais `BCUS98119_INFAMOUS_*` reconhecidos, saves do inFAMOUS 2 rejeitados |
| O app: jogar, ajustes, controles, saves, logs | ✅ bundle macOS gerado, 72 testes passam, a janela inicia o jogo |
| O app: assistente de instalação / rebuild | ⏳ desativado, ainda tem a receita do inFAMOUS 2 |
| O app se atualiza sozinho | ⏳ desativado nesta distribuição |
| Iluminação e SSAO na GPU (o ganho de velocidade do inFAMOUS 2) | ⏳ exige trabalho próprio deste jogo, veja o roadmap |
| Ultrawide com câmera e HUD corrigidos | ⏳ pendente, opção oculta |
| Jogo inteiro | ❓ ainda não jogado além do tutorial |
| Linux e Windows | ⏳ planejado |

Detalhes e próximos passos em [ROADMAP.pt-BR.md](ROADMAP.pt-BR.md). A lista de
recursos trazidos do port do inFAMOUS 2, com o que foi verificado, está em
[PORT_FEATURES.md](PORT_FEATURES.md).

## Baixar e jogar

Ainda não há release. Você precisa de um Mac com Apple silicon e da sua
própria cópia do inFAMOUS (a versão PSN `NPUA80480`, por exemplo instalada no
[RPCS3](https://rpcs3.net) com a licença `.rap`).

1. Compile o jogo a partir da sua cópia, veja [Compilando a partir do código](#compilando-a-partir-do-código).
2. Abra **Jogar inFAMOUS.command** para jogar direto, ou
   **Launcher inFAMOUS.command** para o app (gráficos, controles, áudio, mods
   e saves) e use o botão Jogar.

## O app

[`launcher/`](launcher/README.md) é um app de desktop (Tauri), adaptado do
inFAMOUS 2, com pasta de configurações própria (`infamous1-recomp`), separada
da do segundo jogo. Em português e inglês.

- **Gráficos:** janela, tela cheia ou sem bordas, monitor, vsync, gama, filtragem
  anisotrópica (até 16×), FXAA ou SMAA, nitidez, upscaling MetalFX, resolução
  interna, ajustes de cor.
- **Controles:** teclado, mouse e controle, câmera no mouse, vibração; **áudio**
  com volume e saída.
- **Saves:** listar, fazer backup, restaurar, importar/exportar com o RPCS3,
  `.zip`. Aceita `BCUS98119` / `NPUA80480` e rejeita `BCUS98125` (inFAMOUS 2).
- **Troféus, logs e relatórios**, tudo local; nada é enviado.
- **Pacotes de textura** e gerenciador do cache de shaders.
- **Atalho do Steam** (opcional, só quando pedido na interface).

Ainda não habilitados neste jogo: iluminação e SSAO na GPU, remoção de passes
de shader do inFAMOUS 2 por hash, correção de câmera/HUD ultrawide, o
assistente de instalação/rebuild e a atualização automática. Veja
[launcher/README.md](launcher/README.md).

## Problemas conhecidos

- A campanha completa e sessões longas não estão validadas.
- Iluminação e SSAO ainda rodam nos jobs SPU originais, o que custa fps; o job
  que faz a iluminação está sendo identificado (candidato: image 209).
- Proporções ultrawide esticam o HUD: o patch de câmera é limitado ao hash do
  EBOOT do inFAMOUS 2.
- Na primeira vez que o jogo mostra um efeito novo, o shader é traduzido na
  thread de render, o que pode travar por um instante; o cache faz isso
  acontecer uma vez só.
- Windows e Linux ainda não são suportados.

## Em andamento

O que está sendo feito agora (detalhes em [ROADMAP.pt-BR.md](ROADMAP.pt-BR.md)):

- **Jogar a história inteira**, além do tutorial, e corrigir o que aparecer.
- **Iluminação e SSAO na GPU** para este jogo: achar o job de iluminação,
  mapear buffers e sinais, comparar com os shaders Metal do inFAMOUS 2.
- **Correções dos problemas do jogo original** (quedas de fps, aliasing, pop-in,
  câmera, escalada, combate, IA, checkpoints, texturas): os de imagem e
  velocidade saem das alavancas do runtime, os de lógica exigem patches no
  código traduzido.
- **Um assistente de rebuild e um release** para o app.
- **Windows e Linux:** planejado.

## Compilando a partir do código

### O que você precisa

- Um Mac com Apple Silicon (testado num M4 Pro, macOS 27).
- Uma cópia legítima do inFAMOUS instalada no [RPCS3](https://rpcs3.net), com a
  licença (`.rap`) e o firmware do PS3 instalados nele.
- Xcode Command Line Tools e Homebrew:

```sh
brew install cmake ninja sdl2 glslang spirv-cross spirv-tools python@3.12
python3.12 -m venv .venv
```

### Compilando

```sh
git clone https://github.com/erikomis/infamous1-recomp.git
cd infamous1-recomp
tools/setup_toolkit.sh                        # toolkit em ./ps3recomp-fork
.venv/bin/pip install -r ps3recomp-fork/requirements.txt
```

Depois decifre o EBOOT e o firmware (o `--decrypt` do RPCS3, ou seu próprio
arquivo de chaves no formato scetool com `infamous1-lle/tools/decrypt_self.py`;
nenhuma chave vem neste repositório), traduza e compile:

```sh
python3 infamous1-lle/tools/rebuild.py --dry-run     # lista os passos e confere os pré-requisitos
cd infamous1-lle && cmake -S . -B build -G Ninja && ninja -C build Infamous1Recomp
```

A receita dos módulos de job está em `infamous1-lle/tools/` (`lift_spu_jobs.sh`,
`extract_eboot_jobmods.py`, `lift_datamods.py`, `lift_relocmods.py`).

## Jogando pelo terminal

```sh
cd infamous1-lle
./run.sh
```

O `run.sh` define a janela de memória (`PS3_MEM_ALLOC_SIZE=0x30000000`) e o
caminho de boot HDD (`/dev_hdd0/game/NPUA80480/USRDIR/EBOOT.BIN`) que o jogo
precisa. Um controle (DualSense testado) funciona direto; teclado e mouse
somam por cima, com o mesmo mapa do port do inFAMOUS 2:

| Tecla | PS3 |
| --- | --- |
| W A S D | analógico esquerdo |
| I J K L ou mouse | analógico direito (câmera) |
| Setas | D-pad |
| Espaço | ✕ |
| Q / E / R | ○ / □ / △ |
| U / O | L1 / R1 |
| Y / P | L2 / R2 |
| G / H | L3 / R3 |
| Return / Tab | START / SELECT |

Na tela inicial, **START** (`0x0008`) avança; ✕ não.

Variáveis úteis, compartilhadas com os outros ports deste toolkit:

| Variável | O que faz |
| --- | --- |
| `PS3_ANISO=2\|4\|8\|16` | filtragem anisotrópica (só texturas com mipmaps). Padrão desligado |
| `PS3_AA=fxaa` / `smaa` | anti-aliasing na imagem de 1280×720 |
| `PS3_RES_SCALE=1.5\|2\|3` | resolução interna (1080p / 1440p / 4K) |
| `PS3_UPSCALE=metalfx` | upscaling espacial MetalFX até a janela |
| `PS3_SHARPEN=<0..1>` | nitidez adaptativa ao contraste |
| `PS3_WINDOW_MODE=windowed\|fullscreen\|borderless` | como o jogo abre |
| `PS3_MEM_ALLOC_SIZE=<hex>` | a janela do alocador, de 16 MB a 3 GB (padrão 256 MB; o `run.sh` usa 768 MB) |
| `PS3_ASPECT=21:9` | ultrawide; **pendente neste jogo**, o HUD não está corrigido |

Para testes headless: `tools/capture.sh <nome> <segundos>` salva um PNG do
frame a cada 10 s, `PAD_SCRIPT="145:0x0008"` aperta START aos 145 s, e
`tools/play_prologue.sh` faz o caminho da tela inicial ao tutorial num comando
só. `tools/profile_run.sh` informa o fps por período e o código mais quente.

## Correções feitas aqui

- **Janela de memória (`sys_memory`):** o alocador do toolkit terminava 256 MB
  depois da base, então a varredura do cache de streaming no boot acabava a
  memória e o init de áudio/streaming nunca concluía. O fim da janela agora é
  `PS3_MEM_ALLOC_SIZE`. Foi isso que passou o jogo do boot.
- **Boot por HDD (`cellGame`):** `runtime_overrides/cellGame.c` lê o `CATEGORY`
  do `PARAM.SFO` (`HG` aqui) e informa boot HDD em vez de disco.
- **Limpeza recursiva:** `func_00105550` fazia uma limpeza recursiva sobre a
  instalação enquanto `cellFsUnlink` / `cellFsRmdir` eram no-ops, reabrindo
  `sounds/music` dezenas de milhares de vezes; achado com o trace do sistema de
  arquivos.
- **Retornos SPU:** correções dos retornos SPU nos módulos de job traduzidos
  (`relift_spu_returns.py`).
- **Patches do jogo** (`game_patches.cpp`) são filtrados pelo hash do program
  header (`PPU-<sha1>`), então um patch do inFAMOUS 2 nunca é aplicado neste
  EBOOT.

## Estrutura

| Pasta | O que é |
| ----- | ------- |
| [`infamous1-lle/`](infamous1-lle) | **Port atual.** Fork do inFAMOUS 1 + LLE do firmware: `main.cpp`, `stubs.cpp`, `run.sh`, overrides de runtime, ferramentas SPU/PPU. |
| `infamous1/` | Arquivos locais do jogo (`vfs/`), nunca publicados. |
| [`launcher/`](launcher/README.md) | O app de desktop (Tauri): jogar, ajustes, saves. |
| `tools/` | `setup_toolkit.sh` clona o fork no commit em uso; scripts de lift dos módulos de job. |
| [`NOTES.md`](NOTES.md) | Notas de trabalho e evidências. |
| [`PORT_FEATURES.md`](PORT_FEATURES.md) | O que veio do port do inFAMOUS 2 e o que foi verificado. |

## Licença

Código deste repositório: MIT ([LICENSE](LICENSE)). O toolkit e o fork são MIT
pelos seus autores. inFAMOUS © Sony Computer Entertainment America LLC,
desenvolvido pela Sucker Punch Productions; este projeto não tem vínculo com a
Sony.
