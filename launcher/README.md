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

Atualização automática e instalação/rebuild pelo assistente estão desativados:
o assistente herdado ainda contém a receita de lifting do segundo jogo. O
launcher executa o build existente; não substitui o EBOOT, firmware ou código
recompilado. Para compilar o jogo, use o CMake configurado em `infamous1-lle`.

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
