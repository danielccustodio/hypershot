<p align="center">
  <img src="assets/hypershot.svg" width="96" alt="Ícone do HyperShot">
</p>

<h1 align="center">HyperShot</h1>

<p align="center">
  <strong>Uma ferramenta de captura de tela para Windows, de código aberto e escrita em Rust.</strong><br>
  Selecione um trecho da tela, faça marcações e copie, salve, fixe ou junte a captura em uma imagem longa.
</p>

<p align="center">
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/licen%C3%A7a-MIT%20ou%20Apache--2.0-0d9488.svg" alt="Licença: MIT ou Apache-2.0"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2021-ed5a22?logo=rust&logoColor=white" alt="Rust 2021"></a>
  <a href="https://github.com/danielccustodio/hypershot/actions/workflows/ci.yml"><img src="https://github.com/danielccustodio/hypershot/actions/workflows/ci.yml/badge.svg" alt="Fluxo de CI"></a>
</p>

<p align="center">
  <a href="#comece-por-aqui">Comece por aqui</a> - <a href="#como-usar-o-hypershot">Como usar</a> - <a href="#compilar-e-testar">Compilar e testar</a> - <a href="#contribuir">Contribuir</a> - <a href="README.md">English</a>
</p>

> [!WARNING]
> **Situação atual:** o HyperShot está em desenvolvimento. A experiência completa para quem usa o aplicativo foi implementada para **Windows**: ícone na bandeja, atalhos globais, tela de seleção, ferramentas de edição, salvamento, cópia para a área de transferência, fixação na tela e captura com rolagem. O projeto tem dependências de captura multiplataforma e scripts de empacotamento para Linux e macOS, mas a interface nativa desses sistemas ainda não está pronta para uso. Para usar o aplicativo agora, escolha Windows.

## O que é o HyperShot?

O HyperShot transforma uma parte da sua tela em uma imagem pronta para usar. Ele serve para aquelas situações corriqueiras em que uma captura resolve a conversa: mostrar um erro, explicar um passo, mandar uma referência, esconder uma senha antes de enviar a imagem ou manter uma informação pequena visível enquanto você trabalha.

O fluxo é simples:

1. Você aperta um atalho.
2. Arrasta o mouse sobre o trecho que quer capturar.
3. Se quiser, acrescenta uma seta, uma marcação, um passo numerado ou pixelização.
4. Copia, salva, fixa ou cria uma captura mais longa com rolagem.

Você não precisa saber Rust para usar o programa. Rust é apenas a linguagem usada para criar o HyperShot.

## O que já funciona

| Quando você precisa... | O HyperShot pode... |
| --- | --- |
| Capturar uma parte da tela | Abrir uma tela de seleção e recortar a região escolhida. |
| Chamar atenção para algo | Desenhar com caneta, linha, seta, retângulo ou marca-texto. |
| Mostrar uma ordem de passos | Colocar círculos numerados que avançam automaticamente. |
| Esconder informação visível | Pixelizar uma área da captura. |
| Colar a imagem em outro lugar | Copiar a captura para a área de transferência do Windows. |
| Guardar um arquivo | Salvar em PNG, JPEG ou WebP pela janela padrão de salvamento. |
| Deixar uma referência à vista | Fixar a imagem escolhida em uma janela sem borda que fica acima das outras. |
| Capturar uma página ou conversa longa | Criar uma imagem costurada enquanto você rola a área selecionada. |
| Disparar a captura rapidamente | Usar o ícone da bandeja, `PrintScreen` ou `Ctrl` + `Shift` + `X`. |

O código atual processa as capturas no computador. Ele não contém conta em nuvem, serviço de upload ou sincronização automática de imagens.

## Comece por aqui

### Antes de começar

Para o caminho suportado no Windows, instale duas ferramentas gratuitas:

1. [Git](https://git-scm.com/downloads), que baixa o projeto.
2. [Rust](https://rustup.rs/), que instala o comando `cargo`, usado para compilar e executar o programa.

Depois de instalar o Rust, feche o PowerShell e abra-o outra vez. Assim o Windows passa a encontrar o comando `cargo`.

### Executar o HyperShot a partir do código

Abra o **PowerShell** e execute os comandos abaixo, um por vez:

```powershell
git clone https://github.com/danielccustodio/hypershot.git
cd hypershot
cargo run --release
```

O que cada comando faz:

- `git clone` baixa uma cópia do projeto para uma nova pasta chamada `hypershot`.
- `cd hypershot` leva o PowerShell para dentro dessa pasta.
- `cargo run --release` cria uma versão otimizada do executável, inicia o HyperShot e abre a primeira captura.

A primeira compilação pode levar alguns minutos, pois o Rust baixa e compila as bibliotecas usadas pelo projeto. Nas próximas vezes tende a ser mais rápido, porque o Cargo reaproveita a compilação que já fez.

Depois de iniciado, o HyperShot fica disponível na área de notificação do Windows, perto do relógio. Dependendo da configuração do sistema, talvez seja preciso clicar em `^` para ver o ícone.

### Se algum comando der erro

| O que aparece | O que costuma significar | O que fazer |
| --- | --- | --- |
| `git is not recognized` | O Git não foi instalado ou o PowerShell não foi reaberto depois da instalação. | Instale o Git e abra uma nova janela do PowerShell. |
| `cargo is not recognized` | O Rust não foi instalado ou o terminal ainda não sabe onde encontrá-lo. | Instale o Rust pelo rustup e abra uma nova janela do PowerShell. |
| Um erro de captura aparece no terminal | O HyperShot não conseguiu ler a imagem da tela. | Guarde a mensagem do terminal e use-a em uma issue, sem compartilhar conteúdo sensível da tela. |
| `PrintScreen` não faz nada | Outro programa pode estar usando essa tecla. | Use `Ctrl` + `Shift` + `X`, que também é registrado pelo HyperShot. |

## Como usar o HyperShot

### Fazer uma captura comum

1. Inicie o HyperShot uma vez com `cargo run --release`.
2. Aperte `PrintScreen`. Se a tecla estiver indisponível, use `Ctrl` + `Shift` + `X`.
3. A tela fica congelada atrás da sobreposição de captura.
4. Clique e arraste para desenhar um retângulo sobre o conteúdo que deseja guardar.
5. Se quiser editar a imagem, escolha uma ferramenta na barra vertical.
6. Use a barra horizontal para fechar, salvar, copiar, fixar ou entrar no modo de rolagem.

Também é possível iniciar uma captura clicando com o botão esquerdo no ícone da bandeja. Um clique com o botão direito abre um menu com captura, preferências, pasta de capturas e saída do aplicativo.

### Atalhos durante a seleção

Estes atalhos funcionam enquanto a tela de seleção está aberta:

| Atalho | Resultado |
| --- | --- |
| `Esc` | Cancela a captura. |
| `Enter` | Copia a seleção atual para a área de transferência e fecha a tela. |
| `Ctrl` + `C` | Copia a seleção atual para a área de transferência e fecha a tela. |
| `Ctrl` + `S` | Abre a janela de salvamento. |
| `Ctrl` + `Z` | Desfaz a última anotação. |
| Botão direito, antes de selecionar uma área | Cancela a captura. |

### Ferramentas de edição

A barra vertical contém as ferramentas abaixo. Os nomes descrevem a intenção de uso, não um formato de arquivo nem um modo separado de edição.

| Ferramenta | Use quando quiser... |
| --- | --- |
| Caneta | Fazer uma marca à mão livre. |
| Linha | Desenhar uma linha reta. |
| Seta | Apontar de um lugar para outro. |
| Retângulo | Colocar uma moldura em volta de algo. |
| Marca-texto | Acrescentar um destaque colorido e translúcido. |
| Pixelizar | Esconder uma área retangular, como um token visível ou um endereço de e-mail. Confira a imagem final antes de compartilhar. |
| Marcador de passo | Adicionar círculos numerados para uma instrução ou relato de erro. Cada novo marcador usa o próximo número. |
| Botão de cor | Alternar entre as cores de desenho já incluídas. Ele troca a paleta, não captura uma cor da tela. |
| Desfazer | Remover a anotação mais recente. |

Fora do modo de captura com rolagem, a roda do mouse muda a espessura do traço.

### Salvar, copiar e fixar

Depois de selecionar uma área:

- **Copiar** coloca a imagem pronta na área de transferência. Cole-a em um chat, e-mail, documento ou editor de imagem com `Ctrl` + `V`.
- **Salvar** abre a janela do Windows para escolher o nome do arquivo e PNG, JPEG ou WebP.
- **Fixar** cria uma janela pequena que fica acima das outras. Arraste para movê-la. Clique duas vezes ou clique com o botão direito para fechá-la.
- **Fechar** descarta a captura atual.

Ao salvar uma captura, o HyperShot também pode copiá-la para a área de transferência. Essa opção é controlada pelo arquivo de preferências explicado mais abaixo.

### Capturar uma área com rolagem

A captura com rolagem é útil para uma conversa, documento ou página longa que não cabe em uma tela. O recurso compara imagens seguidas e junta as partes que se sobrepõem.

1. Inicie uma captura comum e selecione a região que quer guardar.
2. Clique na ação **Scroll** da barra horizontal.
3. Role a página debaixo da seleção devagar. Mantenha a mesma região sobre o conteúdo que será unido.
4. Quando a imagem tiver todo o conteúdo necessário, escolha **Salvar** para gravar a imagem costurada atual, ou **Copiar**. A tecla `Enter` também copia a imagem costurada atual e fecha a tela.

Há alguns limites práticos:

- Deixe uma parte do conteúdo anterior visível em cada passo da rolagem. Essa sobreposição permite alinhar as imagens.
- Role devagar o bastante para a tela ser atualizada antes da próxima imagem.
- Vídeos, animações, espaços de carregamento e páginas que mudam depressa podem impedir um encaixe limpo.
- Depois de entrar no modo de rolagem, as ferramentas de desenho não alteram a imagem costurada que será exportada. Faça as anotações antes de ativar esse modo.

## Preferências e arquivos salvos

No Windows, clique com o botão direito no ícone da bandeja e escolha **Editar Preferências**. O HyperShot cria automaticamente o arquivo de configuração na primeira vez em que precisa dele:

```text
%APPDATA%\HyperShot\config.json
```

O arquivo é um texto JSON comum. As configurações padrão incluem:

| Configuração | Padrão | Para que serve |
| --- | --- | --- |
| `save_directory` | `%USERPROFILE%\Pictures\HyperShot` | Pasta sugerida na janela de salvamento. |
| `default_format` | `png` | Formato inicialmente escolhido para novas capturas. |
| `copy_to_clipboard_on_save` | `true` | Copia a imagem depois de salvá-la. |
| `primary_hotkey` | `PrintScreen` | Atalho principal gravado no arquivo. |
| `fallback_hotkey` | `Ctrl+Shift+X` | Atalho alternativo gravado no arquivo. |
| `default_color` | valor RGBA vermelho | Cor inicial para anotações. |
| `default_stroke_size` | `4` | Espessura inicial das linhas em pixels. |
| `enable_loupe` | `true` | Preferência gravada para a lupa de seleção. |
| `dark_backdrop_level` | `150` | Valor gravado para a opacidade do fundo escuro de seleção. |

No momento, os atalhos do Windows são registrados pelo processo de bandeja como `PrintScreen` e `Ctrl` + `Shift` + `X`. Alterar apenas o texto no arquivo JSON ainda não muda esse registro.

## Opções de linha de comando

Na maior parte do tempo, basta iniciar o HyperShot sem opções. Estes comandos existem quando você precisa deles:

| Comando | O que faz |
| --- | --- |
| `cargo run --release` | Inicia o processo de bandeja e abre uma captura imediatamente. |
| `cargo run --release -- --daemon` | Inicia o processo de bandeja do Windows sem abrir uma captura logo em seguida. `--tray` faz a mesma coisa. |
| `cargo run --release -- --demo` | Captura a tela principal, acrescenta algumas anotações de exemplo, grava `hypershot_demo.png` e tenta copiar a imagem. |
| `cargo run --release -- --help` | Mostra a ajuda interna do comando. |

## Compilar e testar

Use estes comandos antes de abrir um pull request ou quando quiser verificar se a sua cópia ainda compila:

```powershell
# Executa os testes automatizados atuais
cargo test --all-targets

# Verifica se todos os destinos compilam
cargo check --all-targets

# Cria o executável otimizado sem iniciá-lo
cargo build --release
```

O repositório inclui um fluxo do GitHub Actions que executa `cargo check`, `cargo test` e uma compilação de lançamento no Windows, Ubuntu e macOS a cada push e pull request.

### Gerar os arquivos de distribuição do Windows

O projeto inclui um script do Inno Setup para criar um instalador do Windows e um arquivo ZIP portátil. Instale primeiro o **Inno Setup 6** e execute:

```powershell
powershell -ExecutionPolicy Bypass -File installer\windows\build_installer.ps1
```

O script compila o executável de lançamento, cria o instalador e coloca o resultado em `dist\`. Os arquivos de distribuição são ignorados pelo Git para não serem enviados ao repositório por acidente.

Também existem scripts de empacotamento em `installer/linux/` e `installer/macos/`. Eles fazem parte do projeto, mas não mudam o status de suporte ao usuário final, que hoje é Windows, descrito no começo deste documento.

## Estrutura do projeto

| Caminho | O que há nele |
| --- | --- |
| `src/main.rs` | Ponto de entrada do programa e tratamento das opções de linha de comando. |
| `src/core/` | Captura de tela, filtros de imagem, primitivas de anotação, cópia para a área de transferência, preferências e costura da rolagem. |
| `src/ui/` | Processo de bandeja do Windows, tela de seleção, barras de ferramentas e janela fixada. |
| `assets/` | Ícones do aplicativo. |
| `installer/` | Scripts de instalador e de pacotes. |
| `scripts/` | Scripts auxiliares para gerar recursos. |
| `.github/workflows/` | Automação de CI e de lançamento. |

## Privacidade e compartilhamento responsável

O HyperShot não envia capturas automaticamente, mas uma imagem de tela pode conter senhas, chaves de API, mensagens pessoais, endereços, dados de clientes ou documentos privados. A pixelização existe para ajudar a esconder uma região escolhida, não para garantir que toda informação sensível desapareceu.

Antes de enviar uma captura:

1. Olhe a imagem inteira, inclusive abas do navegador, notificações, barras laterais e barra de tarefas.
2. Pixelize ou recorte o que não pretende compartilhar.
3. Se a imagem for pública, abra o arquivo salvo e confira uma segunda vez.
4. Nunca publique segredos, tokens de acesso ou capturas privadas em uma issue ou pull request.

## Contribuir

Contribuições são bem-vindas, inclusive de quem vai contribuir pela primeira vez. Melhorias na documentação, relatos de erro claros, testes pequenos, testes no Windows e trabalho rumo ao suporte de Linux ou macOS são todos úteis.

Leia [CONTRIBUTING.md](CONTRIBUTING.md) antes de começar. Um fluxo simples de contribuição é este:

1. Faça um fork do repositório no GitHub.
2. Crie uma branch com um nome curto e descritivo.
3. Faça uma alteração focada em um assunto.
4. Rode `cargo test --all-targets`.
5. Abra um pull request explicando o que mudou, por que mudou e como você testou.

Ao relatar um erro, informe a versão do Windows, o comando usado, a mensagem exata e passos que outra pessoa pode seguir. Remova dados privados de capturas e registros antes de enviar.

## Licença

O HyperShot está disponível sob a [licença MIT](LICENSE-MIT) ou a [Apache License, Version 2.0](LICENSE-APACHE), à sua escolha.
