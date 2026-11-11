# inFAMOUS 1 (NPUA80480) — static recompilation, status e retomada

Status: **bring-up LLE em curso** (modelo do port comprovado do inFamous 2,
`~/Desktop/recomp-infamous-2/infamous2-lle`). Toolkit: `ps3recomp` fork
`NoahBPeterson/ps3recomp` branch `infamous/rebase-v0.13.0` (commit `770660a`),
em `~/Desktop/recomp-infamous-2/ps3recomp-fork` — o mesmo de todos os outros
ports desta máquina. Rodando em macOS/arm64 (Apple M4 Pro), headless.
Última sessão: 2026-11-10.

## Roadmap dos problemas do jogo original — 2026-11-10

A lista de problemas do jogo original levantada pelo Erik (FPS, aliasing,
pop-in, câmera, escalada, combate, IA, missões, checkpoints, texturas) virou
roadmap priorizado com plano técnico por item: **ROADMAP_JOGO.md** (na raiz,
referenciado também pelo PORT_FEATURES.md). Estado já validado até aqui:
aniso 16× ok, ~44 fps no menu/cutscene; candidato do job de luz anotado
abaixo (image 209) para o próximo passo do GPU light.

## Janela do sys_memory — RESOLVIDO — 2026-11-10

O bloqueio do `boot4`/`boot5` (retorno infinito de `sys_memory
allocate(size=0x100000)` — 26.865 tentativas, `ENOMEM` depois de 125 alocadas
até `0x4FF00000`) era a janela do alocador do runtime: `sys_memory.c`
fixava `SYS_MEM_ALLOC_END = base + 0x10000000` (256 MB).

Correção (no toolkit compartilhado,
`ps3recomp-fork/runtime/syscalls/sys_memory.c`): o fim da janela agora é
`PS3_MEM_ALLOC_SIZE` (hex; default mantido 256 MB; permitido 16 MB..3 GB,
alinhado a 1 MiB). O `run.sh` do inF1 fixa `PS3_MEM_ALLOC_SIZE=0x30000000`
(768 MB). Rebuild com `ninja Infamous1Recomp` (o alvo `ps3recomp_sdk` não
existe isolado; o binário recompila o SDK junto).

Evidência —— `out/boot/boot7.log` (headless, 55 s,
`PS3RECOMP_METAL_FRAME_DUMP=1`):

- Zero `null-write`/`328_dust`, zero alocações em loop, só 66 chamadas
  totais, teto final `0x4AC00000` (~(0x400..0x4AC) ≈ 172 MB usados).
- Fluxo completo atingido (que o boot4 nunca viu): Trophy Thread +
  `sceNpTrophy` init → opens dos psarcs
  (`cache/psarc/install1|2/infamous1|2.psarc_s`, magic `PSAR`) →
  `cellAudio Init()` + `PortOpen/PortStart` (port 0 e port 1) →
  `splash_america.bik` (magic `BIKi`) → `tv_default_feed_2x2.bik` →
  mp3s de `sounds/music` e `streaming_grains` → threads
  "SP Movie Loader"/"SP Movie Player"/"Bink Audio"/"Bink IO" →
  "Game State Thread" → 32 pipelines de shader compilados.
- Render confirmado por captura: `frame=2400` com `draws=10171`,
  `surfaces=15` (~85 draws/frame, menu rodando; 236 blocos de captura).

Interpretação revisada (vale para as notas de 2026-11-08): o "não aumentar
simplesmente a janela" estava errado. O objeto `.bss` `0x925AC4` (vtable
NULL, NULL-vcall, strcpy em NULL) era **sintoma**, não causa: com a janela
estourando no meio do scan de cache, o init de áudio/streaming nunca
concluía, então o singleton nunca era construído. O construtor não precisa
ser rastreado — com memória o init roda e constrói.

Validação de gameplay com o novo alocador (`out/start-run/`, 200 s):
`PAD_SCRIPT="95:0x0008,115:0x0008,130:0x4000,145:0x4000"` (START 0x0008 —
não 0x4000, que é CROSS — para sair da title; CROSS confirma o menu)
leva da title screen ao gameplay: Cole no crater do prologue com o
tutorial "Use the right stick to look up at the chopper" renderizado
(`t200.png`), cutscene lava/explosão já rodando. Teste longo do prologue
inteiro: `tools/play_prologue.sh` (timing medido para runs de 2026-11-09;
revisar tempos se o menu ficar mais rápido).

## Launcher — validação do fluxo de janela — 2026-11-10

- Bundle macOS atual (`launcher/src-tauri/target/release/bundle/macos/
  infamous1-recomp.app`, 09:10, 72 testes passando; `npm run build` ok).
- Smoke test do caminho real do launcher (`run.sh` sem HEADLESS,
  `out/window-smoke.log`, 40 s): janela Metal 1280×720 na Retina interna,
  splash .bik lendo, pipelines compilando, `cellSaveData` ativo. O build
  do recomp que o launcher executa já carrega a correção da janela de
  memória (via `run.sh`).
- `tools/play_prologue.sh` reescrito: o script antigo usava CROSS (0x4000)
  na title screen — comprovado que não avança; só START (0x0008). Tempos
  reancorados nas evidências do `out/start-run` (mundo começa 145-200 s:
  stick direito 196-256 s (2 ciclos), esquerdo 264-296 s + frente até o
  fim, aim L1 720 s + bolts R1 730-1130 s).
- `tools/capture.sh`: fallback de PY para o venv do toolkit/`python3` (a
  linha antiga apontava para um `.venv` que não existe neste projeto).
- Candidatos a job de iluminação (PORT_FEATURES item 1 "identificar os
  jobs"): overlays streamados para LS 0x4000 durante boot/attract (e
  provavelmente alguns deles em gameplay): image 209 src 0x61E680 (13×),
  215 src 0x6E8E00 (7×), 218 src 0x629100 (6×), 213 src 0x65A100,
  207 src 0x667280, 200 src 0x669780, 201 src 0x64DE00. Confirmação
  pendente: qual deles escreve o label/protocolo que o RSX amostra por
  tile (o padrão do inF2 era image 219 + label 135).
- **Filtragem anisotrópica até 16× — validada no inF1** (`out/aniso-smoke.log`,
  `out/aniso16/`): o launcher já expõe 0/2/4/8/16× (UI + clamp a 16
  em `launcher/src-tauri/src/lib.rs:684`), o runtime honra `PS3_ANISO`
  (`rsx_metal_backend.m` aplica só em samplers com minificação linear e
  mipmaps) e o inF1 loga
  `[RSX metal] graphics: anisotropic 16x, ...` no boot. Frame da title com
  16× sem regressão visual (`t030.png`).
- **Job de luz do inF1 — candidato líder identificado** (2026-11-10):
  image **209** = `jobmod_6cbbde2f1bfc_e9d68` (entry 0xDD68, bin 43.616 B,
  src streamado `0x61E680` → LS 0x4000) é o overlay mais streamado em
  boot/cutscene (13×), o mesmo perfil do job de luz do if2 (image 219 =
  `jobmod_c6e7383dc013_e5400`, também o mais streamado). Alternativas:
  image 215 (`jobmod_cbe85abb417c_e12058`, src 0x6E8E00, 6×) e image 218
  (`jobmod_d46637430ef9_e5bb8`, src 0x629100, 6×). Nenhum hash de jobmod é
  compartilhado com o if2 (recompilados entre jogos; similaridade por
  n-grams não separa os módulos — boilerplate WWS domina). Confirmar com o
  teste A/B do padrão do if2: `spu_replace_function` no-op na rotina de
  tile do candidato e observar o efeito visual por frame (no if2, job de
  luz quebrado = props glossy estourando branco / luz nas cores das
  normais). Mapeamento completo image→jobmod em
  `spu/jobmods_register.c` (200-220).

## Launcher e comparação de recursos — 2026-11-10

Launcher Tauri do inFAMOUS 2 adaptado em `launcher/`, com título, caminhos,
identificadores e configurações próprios. Build macOS gerado; 72 testes
passaram (3 opcionais ignorados). O teste `--play` iniciou a janela Metal
no M4 Pro com `BootCheck: type=2, titleId=NPUA80480`.

Os saves do primeiro jogo foram confirmados na instalação local como
`BCUS98119_INFAMOUS_0`; o gerenciador aceita BCUS98119/NPUA80480 e rejeita
BCUS98125/INFAMOUS2. Não foram alterados saves originais.

Atalho: `../Launcher inFAMOUS.command`. Recursos e limitações estão em
`../PORT_FEATURES.md`. GPU light/SSAO e ultrawide do segundo jogo continuam
desabilitados; não foram anunciados como portados. O scanner de relocmods
do segundo jogo encontrou zero blobs neste EBOOT. O assistente de rebuild
herdado e o updater permanecem desativados nesta versão local.

## Avanço validado — 2026-11-09

O recomp chegou à **tela inicial 3D "Press the Start Button"** no Metal,
1280×720, aceitou Start e avançou pela explosão até **Cole no cenário com
o tutorial "Use the right stick to look up at the chopper"**. A campanha
completa e a estabilidade de sessões longas não foram validadas.

Correções desta sessão:

1. `runtime_overrides/cellGame.c` lê CATEGORY do PARAM.SFO (`HG` neste jogo)
   e informa boot HDD, em vez do antigo DISC incondicional. `run.sh` passa
   `PS3_BOOT_PATH=/dev_hdd0/game/NPUA80480/USRDIR/EBOOT.BIN`.
   Teste isolado `tests/test_game_boot.c`: passou para tipo e content paths.
   A mesma correção de CATEGORY existe no toolkit do Afro Samurai.
2. Identificada a causa da memória: `func_00105550` executava limpeza recursiva
   sobre a instalação, enquanto `cellFsUnlink/Rmdir` eram no-ops. O trace
   `out/boot/fs-trace.log` mostrou mais de 54 mil reaberturas de `sounds/music`.
   O alocador de arena aproveitava normalmente cada página (~916 strings por
   32 KiB); aumentar a memória ou alterar o mapper não era a solução.
   Com boot HDD o jogo deixou essa rotina e abriu `splash_america.bik`.
3. A imagem SPU `spu_0003_at_006A4E80` precisava de pontos de retorno registrados
   (primeiros misses: `0x18BB0`, `0x18DB0`). Relift com seus 464 `spu_link` PCs
   removeu os aborts SPURS observados e permitiu tocar a vinheta e renderizar
   o menu. Receita reproduzível: `tools/relift_spu_returns.py --toolkit <SDK>`,
   usando Python 3.10+, seguido de build. O jobmod duplicado
   `jobmod_542edebc6499_e80` também recebeu a entrada `0x18BB0`, mas foi o relift
   da imagem 4 que alterou o resultado em runtime.

Evidências: `out/boot/hdd-boot.log` (primeiro abort SPU depois da correção HDD),
`out/boot/returns-boot.log` (45 s, mais de mil frames, sem o abort anterior),
`out/boot/returns-frame.png` (tela inicial 3D). Teste de partida:
`out/gameplay-probe.log` e capturas em `out/gameplay-probe/`; `current.png`
mostra Cole e o tutorial. Start enviado aos 45 s; Cross aos 55 e 65 s.

Para abrir em janela: duplo clique em `../Jogar inFAMOUS.command`, ou `./run.sh`.
Teclado padrão: Enter = Start, Espaço = Cross, WASD = analógico esquerdo,
IJKL = analógico direito. Controle conectado também é atendido pelo runtime.

As hipóteses de bring-up mais abaixo são históricas. Nenhum arquivo do
toolkit compartilhado nem da instalação original no RPCS3 foi alterado.

## Revisão comparativa dos ports — 2026-11-08

Esta seção atualiza as hipóteses antigas abaixo; ainda não há boot jogável comprovado.

- **WWS já registrado:** `stubs.cpp` registra EA `0x0069E400`, LS `0xA00`,
  tamanho 11648, imagem 9, correspondente a `spu/spu_workloads.c`.
  Portanto a ausência de registro descrita abaixo não é mais o estado atual.
- **RESOLVIDO (2026-11-10):** a causa era a janela de 256 MB do
  `sys_memory` do runtime (`ENOMEM` em loop); ver a seção "Janela do
  sys_memory" lá em cima. Nota: a recomendação abaixo de "não aumentar
  simplesmente a janela" estava errada para este caso.
- **Prioridade: investigar o alocador antes de atribuir a trava às SPUs.**
  `out/boot/boot4.log` mostra alocações de 1 MiB até `0x4FF00000`, seguidas
  da escrita NULL em `func_00448C6C`. O runtime usado limita essa janela a
  `0x50000000` e retorna ENOMEM ao excedê-la. `func_00362B80` usa o retorno
  de uma chamada virtual como destino da cópia, sem testar NULL. Isso é
  compatível com falha de alocação; a causa das solicitações sucessivas
  ainda precisa ser rastreada (`PS3_ALLOC_BT=1` já existe no runtime).
  O `sys_memory_free` atual libera o registro, mas não reutiliza a faixa
  do bump allocator. Ainda não foi demonstrado que frees expliquem este caso.
- **Afro Samurai:** README e `ps3recomp-afro` documentam correções de
  sobreposição/reutilização no memory mapper (`PS3_MMAPPER_TRACK`,
  `PS3_MAPPER_WIDE`) e mempools. São referências para auditar a memória;
  não são uma correção comprovada para este caminho `sys_memory_allocate`.
- **inFAMOUS 2:** o build do 1 já usa seu `ps3recomp-fork`. O fallback
  `SPU_INTERP_UNLIFTED` já está presente e habilitado por padrão no código
  desse runtime. Reimportar o toolkit ou copiar endereços do 2 não resolve
  automaticamente a falha. Seus scripts de relift e diagnóstico continuam
  úteis, mas cada módulo precisa corresponder ao EBOOT do 1.
- **Resistance 2:** referências úteis: layout r4 do WWS, IDs de workloads
  por instância, wake-up de LFQueue e retirada de overlays antigos. Auditar
  apenas se os traces do 1 passarem por esses caminhos; este port usa libsre
  LLE, portanto correções do despachante HLE podem não se aplicar.
- **Demon's Souls / Deadpool:** referências para diagnóstico de threads,
  fallback de SPU e ciclo de vida de tarefas. Não foi identificada nelas uma
  correção direta para a cópia em NULL do 1.

(Desfecho em 2026-11-10: ver a seção "Janela do sys_memory" no topo — a
janela de 256 MB era o que prendia o boot; com 768 MB não há loop, o
"328_dust" desapareceu e o menu renderiza. As hipóteses abaixo ficam como
histórico da investigação.)

## O que já foi feito

1. **EBOOT decifrado** em `game/EBOOT.elf`
   (`rpcs3 --decrypt` escreve o ELF ao lado do `.BIN`; o `--decrypt` nativo
   CLI funciona, saiu exit 137 na primeira tentativa por GC init, na segunda
   ok). SHA256:
   `5e22402aa3ffe695106abc1a9956fa3ea78e4977c32d80b37b6db4c3ba7110c7`.
   APP_VER **01.00** (inF2 testado: 01.03).

2. **PPU loader** (`scripts/step-1-ppu-loader.sh`): entry OPD `0x771C30` →
   código `0x10230`, TOC `0x78C0C0`; 8,480 funções únicas (9,364 OPDs);
   198 imports de firmware em 13 bibliotecas (mesmo perfil do inF2);
   código termina em `0x55EB44` (= última stub `0x55EB24` + stride `0x20`;
   descoberto por comparação com o comando do 2, `--code-end 0x742E18`).
   `out/EBOOT.functions.json`, `.imports.json`, `.image.json`, `.loader.json`.

3. **PPU lifter** (`scripts/step-2-ppu-lifter.sh`): `--code-end 0x55EB44`,
   `--max-chunk-lines 150000` → **27,305 funções** em 18 pedaços
   `recompiled/ppu_recomp_000..017.cpp` (5 deviativas unsupported).

4. **SPU images** (`scripts/step-3-spu-images.sh`): `extract_spu_images.py`
   achou **8 SPU ELFs** (inF2 tinha 2): `spu_0000@0x5F7B80`,
   `0001@0x675A00`, `0002@0x691480`, `0003@0x6A4E80`, `0004@0x6C0B00`,
   `0005@0x6D8D80`, `0006@0x6F3480`, `0007@0x701C80`. 
   `extract_eboot_jobmods.py` achou **21 módulos `C0DEC0DE`**
   (inF2: 27) → `spu/jobmods/jobmod_*.bin`; `lift_jobmods.py` →
   **2886 instructions unsupported** espalhadas (5 módulos têm uns pocos);
   `inF2` usava `relift_jobmod.py` para alguns módulos com entry points extra
   que foram descobertos no runtime — para o 1 ainda não fizemos isso.
   Nenhum módulo `DA7ABA5E` (inF2: 5, `lift_datamods.py` → não se aplica).

   Portanto: `build_spu_workloads.py --images spu/images --lifted spu --out
   spu/spu_workloads.c --register-fn infamous1_spu_register_all --constructor
   --title infamous1 --eboot game/EBOOT.elf` — 8 imagens registradas.

5. **stubs.cpp**: o WWS job policy module é passado dinamicamente
   (`CellSpursWorkloadAttribute.pm`), não há constante no EBOOT (inF2 tinha
   `#define WWSJOB_PM_EA 0x008E5B00` / tamanho 11904 / image id 3). Registro
   do PM **está comentado** — precisa ser descoberto no runtime.

6. **Firmware LLE**: `lle/` e `lle_spu/` do if2 (liblv2, libsysmodule, libsre
   levantados do `dev_flash` 4.93 do próprio RPCS3 do usuário) e `fw/`
   (`*.prx`) copiados — levantamento de firmware é independente de jogo.

7. **VFS**: `infamous1/vfs/` = clone copy-on-write do
   `dev_hdd0/game/NPUA80480` (a `.rap` `UP9000-NPUA80480_00-TRIALSHP06091754.rap`
   está em `exdata`, o jogo roda no RPCS3 com ela).
   Apenas corrigido: `rm -f USRDIR/EBOOT.BIN` (o jogo aqui usa `EBOOT.elf`).

## Como rodar

```sh
cd ~/Desktop/infamous1-recomp/infamous1-lle
./run.sh              # janela (Metal)
HEADLESS=1 ./run.sh   # headless (logs em out/boot/bootN.log)
```

## Onde chegou o bring-up

Log: `out/boot/boot1.log` (e `boot2.log`, com `PS3_THREAD_DUMP=55`,
e `boot3.log` mesmo resultado).

Saúde:
- liblv2 LLE carrega e roda (id=0x23000100), sysmodule/gcm/audio/io HLE,
  libsre LLE (id=0x23000700) — kernels de SPURS rodando em lifted
  (image 100, 6 threads em 2 grupos).
- CRT ok; threads `SpursHdlr0/1`×2, `Job Manager Event Handler`×2,
  9 threads FIOS.
- **FIOS 1.3.0 inicializa**; `cellGame DataCheck/BootCheck` ok
  (`/dev_bdvd/PS3_GAME` mapped, titleId `NPUA80480`, APP_VER 01.00).
- **Metal RSX backend inicia** (1280x720); surface 0 registrada.
- 6×sys_memory allocate 0x100000 (arena do jogo).

## Onde trava

A main thread escreve em NULL no meio de uma `strcpy` (guest-fn
`0x00448C6C`) escrita dentro de um wrapper de **virtual call**
(`func_00362B80`: indirect-call + call to `func_00448C6C`). A sstring sendo
escrita é `"328_dust…"` (endereço `0x0FEFF44F`, r4, na stack humana) —
parece um nome de *sfx mount / file* construído em runtime.
Da mesma hora, vtm `ARG[0x40003450]` / `ARG[0x40003E80]` → **vtbl=0x0**,
method code `0x00000000` (descriptors de **worker body** de FIOS — o
callback não foi montado) = a saída *tal vez*: dead worker callback.

Da mesma hora: as 6 SPU threads kernel ficam em **`ch-wait` espera por
evento mask `0x400`** para sempre (nenhum workload SPURS nunca chega).
Isso é exatamente o sintoma do **Wws_Job policy module não registrado** no
port do inF1 (stubs.cpp vazio): `cellSpursAddWorkloadWithAttribute`
(spid `0x55D524`) chamou mas o runtime SPURS do toolkit só atende o
despacho quando o PM é conhecido — no port do inF2 isso era registrado em
stubs.cpp (a disciplina em `spu/images/wwsjob_pm_at_008E5B00.elf` + id 3).

Correlação: os SIX cn soms (no caso, spurs HLE) — algumas
informações levavam ao mesmo tempo a "main thread process file??".

## Próximas coisas a fazer (em ordem)

**A — Retirar o policy module WWS do inFamous 1** (a maior trava):

  1. Rodar o jogo com `PS3RECOMP_ENG_TEXLOG=1`, `SPU_REJOIN_DBG=1`,
     `PPU_HOTMAP=1` e/ou `PS3_MEMDUMP_AT=<ea>:<len>:<sec>:<file>`.
     O que o do inF2 revelou (como fazer está documentado no README dele).
  2. Alternativa instrumentação: `tools/lldb_barrier.py` do if2 (imprimir
     as lanes de barrier) e `toolkit` para mapear callers de
     `cellSpursAddWorkloadWithAttribute` (stub `0x55D524`, chamadas em
     `ppu_recomp_007.cpp` linhas ~140380–140500; 2 call sites
     `lr=0x0041E4F0` e `lr=0x0041E5E0` dentro de `func_0041D470`).
     O ponteiro do PM é passado no campo `pm` da struct
     `CellSpursWorkloadAttribute` (args clássicos: revision `0x220002`,
     sdkVer, `pm`, `sizePm`, `data`, priority) — lido do *stack* e
     dinâmico de uma tabela; o valor esperado: ponteiro raw ~11–12 KB
     (inF2: 11904 bytes) numa página EBOOT (inF2: 0x8E5B00), com a
     sequência de regs de`; instr. prologue 4×2 `il/ila`).
  3. Na descoberta: `python3 spu/lifter.py` para o mm `wrap_spu_elf.py
     --base 0xA00 --entry 0xA00 --out
     spu/images/wwsjob_pm_at_0xXXXXXX.elf`, e registrar
     `spu_overlay_register_region_at(EA, 0xA00, size, <id>)` em stubs.cpp
     (id = o que `build_spu_workloads.c` dá, se 9º adicionado: id 8).

**B — Relift dos SPU job modules com entries extra** (o 2 fez
 `relift_jobmod.py --extra` para módulos com entry-point não óbvio;
 no `inF1` precisa setar `flow.local lift` nos módulos com 0x…, após A).

**C — Deserialize do asset / FIOS "328_dust" toast → decodificar o
 `fios` mount de somas; e try: `PS3_VFS_ROOT` axis... 
 (em inF2 a ordem final era: mpmarts → `Wws_Job` workloads → menu. Aqui
 faltam vários: `audit_entry_calls.py`, `spbalance.py`, `toolkit` "review").

## Log de decisões

- `ne.bv` (magic `BVER`) existe como **arquivo real** no install de disco
  BCUS98125_INSTALL do inF2; o install digital `NPUA80638` também não o
  tinha. O fork `open FAILED` em `USRDIR/ne.bv` do inF1 é **benigno**
  (o port do inF2 registrou o mesmo sintoma: `download_manifest.dat` open
  FAILED e foi adiante). Um `ne.bv` foi fabricado a partir do modelo do
  inF2 (`RETAIL.build.0410.1007.ship`) em
  `infamous1/vfs/dev_hdd0/game/NPUA80480/USRDIR/ne.bv` — manter, não
  documentado como resolver o NULL crash (não resolveu).
- `stubs.cpp` com o PM de inF2 (`0x008E5B00/11904/id3`)**comentado** — não
  rodar com constantes do inF2.
- `run.sh`: cuídos defaults do inF2 (`PS3RECOMP_ENG_GPU_LIGHT=<addr>`,
  `GPU_SSAO=<target>` e `PS3_ASPECT_HUD_AFTER=<hash>`) foram comentados;
  não rodar com endereços de inF2.
- `gpu_light.c` / `gpu_ssao.c`: transformados em no-op placeholder
  (no inF2, hooks Metal GPU que referenciam símbolos `jobmod_*` do inF2;
  `gpu_light_install`/`gpu_ssao_install` se mantêm como API em main.cpp).
- `game_patches.cpp` seguro de rodar: filtragem por hash de program
  headers (PPU-<sha1>) — não existe patch do inF2 no inF1, não aplica.

## Arquivos dedo-duro

- `out/boot/boot1.log` / `boot2.log` / `boot3.log` — logs de bring-up.
- `lift.log` — saída do PPU lifter.
- `lift_jobmods.log` / `lift_images.log` — saídas SPU.
- `out/EBOOT.functions.json` / `.imports.json` / `.image.json` /
  `.loader.json` — dados do loader.

## Datas & proveniência

- Firmware: `dev_flash` 4.93 do RPCS3 do jogador (mesma fonte do inF2);
  `.sprx` decifrados em `fw/`, imagens prx em `fw/out/`.
- `.rap` `UP9000-NPUA80480_00-TRIALSHP06091754.rap` (exdata do jogador, em
  `~/Library/Application Support/rpcs3/dev_hdd0/home/00000001/exdata`),
  tipo licença 2 local (NPDRM); não é redistribuída para o repositório.
  `self_keys.txt` (chaves scetool no formato, para `tools/decrypt_self.py`)
  foi montado em `~/Desktop/infamous1-recomp/self_keys.txt` a partir do
  `deadpool-recomp/tools/self_keys.txt` (+ blocos duplicados como
  `self_type=NPDRM`); no fim o `rpcs3 --decrypt` funcionou e o caminho
  pycryptodome ficou comprovado como alternativa sem RPCS3.
- NID: `cellSpursAddWorkloadWithAttribute` = `0xc0158d8b`, stub
  `0x55d524`; `_cellSpursWorkloadAttributeInitialize` = `0xefeb2679`,
  stub `0x55d5a4`. (todos via `tools/nid_database.py` do fork)
