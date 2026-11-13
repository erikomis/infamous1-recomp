# Launcher local do inFAMOUS 1

Adaptado do launcher Tauri do projeto local `recomp-infamous-2`, com a licença
MIT e os créditos preservados na raiz. Este aplicativo controla o recomp já
compilado nesta pasta. Não inclui os arquivos do jogo.

Abra `../Launcher inFAMOUS.command`. O botão Jogar usa `infamous1-lle/run.sh`.
A instalação é detectada pelo PARAM.SFO e pelo executável deste projeto.
Configurações, controles, backups e relatórios usam `infamous1-recomp`, separados
completamente dos dados do launcher do segundo jogo.

Recursos adaptados: opções de janela e monitor, resolução interna, MetalFX,
FXAA/SMAA, filtro anisotrópico, ajustes de cor, áudio, controles e mouse,
vibração, mods, cache de shaders, backups/importação/exportação de saves,
troféus locais, logs e relatórios. Ações como exportar saves ao RPCS3 ou
adicionar o atalho ao Steam só ocorrem quando solicitadas na interface.
Os saves locais do primeiro jogo usam `BCUS98119_INFAMOUS_*`; os de
`BCUS98125_INFAMOUS2_*` são rejeitados.

Não habilitados: iluminação GPU e SSAO GPU específicos do segundo jogo,
remoção de passes por hashes do segundo jogo e correção de câmera/HUD ultrawide.
Esses recursos exigem mapear e validar o código do primeiro jogo. O renderizador
continua usando os trabalhos SPU originais para iluminação e SSAO.

A instalação/rebuild pelo assistente roda o `infamous1-lle/tools/rebuild.py`,
já portado para este jogo (2026-11-11: numa pasta limpa ele reproduziu o build
testado byte a byte e o binário bootou até o tutorial). A tela em si ainda não
foi testada clicando. O app procura o script em `INFAMOUS1_REPO`, ao lado do
checkout (`../infamous1-lle/tools/rebuild.py`) ou nos recursos do bundle
(`port/`, montados por `scripts/stage_port.py`). A atualização automática
continua desativada.

## RecompLauncher

O [RecompLauncher](https://recomplauncher.com) (um launcher de terceiros)
instala recomps a partir dos zips das releases do GitHub. O perfil que ele lê
é o `recomplauncher.json` na raiz do repositório, que vai no topo do zip ao
lado do `.app`:

- **A cópia** (`game`): o inFAMOUS da PlayStation Store do próprio jogador
  (NPUA80480), a pasta `dev_hdd0/game/NPUA80480` que o RPCS3 instalou, ou a
  pasta de dados do RPCS3 acima dela (o perfil a reconhece pelo TITLE_ID,
  `ids`), com a licença (`.rap`) no `exdata` ou ao lado da pasta. O disco
  (BCUS98119) é recusado com uma mensagem própria.
- **Preparação** (`prepare`): `infamous1-recomp.app/Contents/MacOS/infamous1-launcher
  --prepare --game <cópia>`, com limite de 2 horas. Roda sem janela e faz o
  que a tela Instalar faz: confere a cópia (só leitura: `PARAM.SFO` com
  TITLE_ID NPUA80480 e `USRDIR/EBOOT.BIN`), acha a licença como a tela
  (`search_licence`; senão a que está nas configurações), guarda as duas no
  `launcher.json`, confere as Command Line Tools (o instalador da Apple é
  aberto) e roda o `rebuild.py` com os mesmos argumentos da janela
  (`rebuild_command`) para `<data>/install`, ou para a pasta de instalação
  das configurações quando um rebuild já escreveu lá. Imprime linhas
  `progress <0..1> <texto>` (os passos e pesos de `src/progress.ts`, a partir
  das linhas PLAN/PROGRESS/SUBPROGRESS do `rebuild.py`, sem nunca voltar, com
  o tempo movendo um passo longo) e a saída do `rebuild.py` como linhas
  normais, também em `<data>/logs/install.log`. Sai com 0 só quando o jogo
  está compilado, e então aponta a pasta de instalação das configurações para
  ele. Um por vez (`<data>/prepare.lock`). Rodar de novo atualiza a
  instalação e continua uma que parou.
- **Jogar** (`start`): `--play --game <cópia>`. Com `RECOMPLAUNCHER=1`
  (o RecompLauncher define) a janela não abre: a cópia é guardada, o Jogar
  da janela roda (configurações, backup dos saves, `logs/game.log`,
  `run_sh_command`) e o processo do launcher vira o do `run.sh` (`exec`), que
  dá `exec` no jogo, então o RecompLauncher vê o código de saída do próprio
  jogo e o Parar dele chega ao jogo. Sem essa variável, `--play` abre a
  janela e joga quando ela está pronta, como antes; ali o `--game` é guardado
  antes (um errado é informado e o jogo não começa).
- **Configurações** (`config`, `settings`): modo e tamanho da janela, V-Sync,
  resolução, upscaling, anti-aliasing, filtro de textura, nitidez, idioma,
  ícones dos botões, volume, vibração, gatilhos adaptáveis, barra de luz, mira
  por giroscópio e Mostrar FPS, nos mesmos grupos do port do inFAMOUS 2, gravados no `launcher.json` (as
  mesmas chaves e valores da janela; um teste confere que cada um volta igual
  ao ser lido).
- **Saves e logs**: `<data>/gamedata/dev_hdd0/home/00000001/savedata` e o
  `input.toml`; a pasta de logs é `<data>/logs` (`game.log`, `install.log`).

Códigos de saída do `--prepare` e do `--play` sob o RecompLauncher
(`prepare.rs`, `Exit`; o `exit_codes` do perfil diz cada um ao jogador,
palavra por palavra, o que um teste confere):

| Código | Significado |
|---|---|
| 0 | prepare: o jogo está compilado; jogar: saiu normalmente (`"ok"`) |
| 40 | sem cópia, não encontrada, não é NPUA80480, ou incompleta |
| 41 | o jogador precisa fazer algo antes: Command Line Tools (o instalador abre), ferramentas de build, RPCS3 ou o firmware, ou não é um Mac |
| 42 | um download do build falhou (o toolkit, os pacotes Python: a conexão) |
| 43 | um passo do rebuild falhou (`install.log`), ou falta espaço |
| 44 | outro prepare, um rebuild aberto pela janela, ou o jogo deste port já está rodando |
| 45 | jogar: o jogo ainda não está compilado |
| 46 | o launcher falhou sozinho (as configurações dele, iniciar um programa) |
| 47 | a cópia está certa, mas a licença (`.rap`) não foi encontrada (no `exdata` do RPCS3 nem ao lado da pasta do jogo) |

"Já rodando" é só este jogo: um processo cujo programa é o executável em
`infamous1-lle/build/` (o do bundle ou o simples), não o nome sozinho (os
outros ports também rodam neste Mac) nem um comando que só cita o caminho.

Para testar sem tocar na sua instalação: `HOME=<pasta de rascunho>` move
`<data>` e a pasta do RPCS3 para lá. Uma cópia recusada, `--prepare` sem
`--game`, um lock em `<rascunho>/Library/Application Support/infamous1-recomp/prepare.lock`
ou a falta do RPCS3/firmware ali (o `rebuild.py` para nas checagens) saem
antes de qualquer build. Para o `--play`, aponte antes o `installDir` do
`launcher.json` de rascunho para uma pasta vazia: o padrão de um build de
desenvolvimento é este checkout.

**O zip**: `scripts/package-recomplauncher.sh [--app <.app>] [--out <pasta>]`
monta o `port/` (`scripts/stage_port.py`), compila o `.app` de release com o
`src-tauri/tauri.bundle.conf.json` (ou usa um), confere que o programa está
onde o perfil diz, que é arm64, que tem o `port/infamous1-lle/tools/rebuild.py`
e que o `scripts/check_bundle.py` passa, e escreve
`infamous1-recomp-<versão>-macos-arm64.zip` (o `.app` e o
`recomplauncher.json` no topo) em `launcher/release/`. Não envia nada.
Publicar uma release com esse zip e pôr o jogo na lista do recomplauncher.com
são passos manuais à parte; o badge do README aponta para o recomplauncher.com
até lá.

## Desenvolvimento

```
npm run build
cargo test --offline --manifest-path src-tauri/Cargo.toml --lib
CARGO_NET_OFFLINE=true npm run tauri -- build --bundles app
```

O bundle fica em `src-tauri/target/release/bundle/macos/infamous1-recomp.app`.
Os testes usam diretórios temporários para exercitar backups, restauração,
ZIPs, controles, ambiente de execução e rejeição de saves de outros jogos.
A compilação offline pressupõe as dependências já presentes no Mac.
