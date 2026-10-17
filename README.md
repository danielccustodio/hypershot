<p align="center">
  <img src="assets/hypershot.svg" width="96" alt="Ícone do HyperShot">
</p>

<h1 align="center">HyperShot</h1>

<p align="center">
  <strong>Uma ferramenta aberta de captura de tela para Windows, feita em Rust.</strong>
</p>

<p align="center">
  Se você consegue usar <code>PrintScreen</code>, já sabe começar: selecione uma área, faça anotações e copie ou salve a imagem.
</p>

<p align="center">
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/licen%C3%A7a-MIT%20ou%20Apache--2.0-0d9488.svg" alt="Licença: MIT ou Apache-2.0"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2021-ed5a22?logo=rust&logoColor=white" alt="Rust 2021"></a>
  <a href="https://github.com/danielccustodio/hypershot/actions/workflows/ci.yml"><img src="https://github.com/danielccustodio/hypershot/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
</p>

<p align="center">
  <a href="#começar-agora">Começar agora</a> · <a href="#como-usar">Como usar</a> · <a href="#para-quem-quer-contribuir">Contribuir</a> · <a href="#english-summary">Em inglês</a>
</p>

> [!WARNING]
> **Status honesto:** HyperShot está em desenvolvimento. A experiência completa de captura — bandeja do sistema, atalhos globais e sobreposição de seleção — está implementada para **Windows**. O projeto usa componentes multiplataforma para captura e inclui automações de empacotamento para Linux e macOS, mas a interface nativa desses sistemas ainda não está pronta para uso.

## O que é isto?

HyperShot é um aplicativo para transformar uma parte da sua tela em uma imagem pronta para compartilhar. Ele foi criado em Rust e o código está aberto para qualquer pessoa estudar, usar, melhorar e redistribuir nos termos da licença.

Em linguagem simples: pressione um atalho, arraste o mouse sobre o que quer recortar, desenhe se quiser e então copie, salve ou fixe a imagem na tela.

## O que já funciona

| Você quer… | O HyperShot faz… |
| --- | --- |
| Recortar uma área da tela | Abre uma sobreposição para selecionar a região com o mouse. |
| Marcar algo importante | Disponibiliza caneta, linha, seta, retângulo, marca-texto e marcadores numerados. |
| Esconder uma informação | Aplica pixelização à região escolhida. |
| Mostrar um passo a passo | Adiciona marcadores numerados sequenciais. |
| Copiar e colar em outra conversa | Copia a captura para a área de transferência do sistema. |
| Guardar como arquivo | Abre a janela para salvar em PNG, JPEG ou WebP. |
| Manter uma referência visível | Fixa o recorte em uma janela sempre visível. |
| Capturar uma área longa | Inicia o modo de rolagem, que junta os quadros capturados enquanto você rola. |
| Capturar sem abrir o terminal toda vez | Mantém um ícone na bandeja do Windows e registra atalhos globais. |

As imagens são tratadas localmente. O código atual não inclui um serviço de upload ou sincronização em nuvem.

## Começar agora

### Opção A — executar pelo código-fonte (recomendado por enquanto)

Você precisa de um computador Windows, do [Git](https://git-scm.com/downloads) e do [Rust](https://rustup.rs/) instalados.

1. Abra o **PowerShell**.
2. Copie e cole estes comandos, um de cada vez:

   ```powershell
   git clone https://github.com/danielccustodio/hypershot.git
   cd hypershot
   cargo run --release
   ```

3. A tela será preparada para a primeira captura e o HyperShot continuará disponível na bandeja do sistema.

Se o Git não estiver instalado, instale-o pelo [site oficial](https://git-scm.com/downloads). Se `cargo` não for reconhecido, instale o Rust pelo [rustup](https://rustup.rs/) e abra o PowerShell novamente.

### Opção B — gerar um executável local

Se você quer apenas o arquivo do aplicativo depois de compilar:

```powershell
cargo build --release
```

O executável será criado em `target\release\hypershot.exe`.

O repositório também contém scripts de empacotamento. Para criar um instalador e um pacote portátil do Windows localmente, é necessário ter o **Inno Setup 6** instalado:

```powershell
powershell -ExecutionPolicy Bypass -File installer\windows\build_installer.ps1
```

## Como usar

### Captura comum

1. Com o HyperShot em execução, pressione `PrintScreen`.
2. Se esse atalho estiver ocupado por outro programa, use `Ctrl` + `Shift` + `X`.
3. Clique e arraste para selecionar a parte desejada da tela.
4. Use a barra vertical para escolher uma ferramenta de desenho.
5. Use a barra horizontal para copiar, salvar, fixar ou iniciar uma captura com rolagem.

Atalhos úteis dentro da tela de seleção:

| Atalho | Ação |
| --- | --- |
| `Esc` | Cancela a captura. |
| `Ctrl` + `C` ou `Enter` | Copia o recorte para a área de transferência. |
| `Ctrl` + `S` | Abre a janela para salvar o recorte. |
| `Ctrl` + `Z` | Desfaz a última anotação. |

### Captura com rolagem

1. Selecione a área que deseja capturar.
2. Clique na ação de rolagem na barra horizontal.
3. Role a página ou conversa devagar, mantendo a mesma área visível.
4. Pressione `Enter` quando terminar e salve ou copie o resultado.

Esse recurso depende de haver sobreposição visual suficiente entre os quadros. Em páginas com animações, vídeo ou mudanças muito rápidas, o encaixe pode falhar.

### Preferências

No Windows, clique com o botão direito no ícone do HyperShot na bandeja do sistema e escolha **Editar Preferências**. O arquivo é salvo em:

```text
%APPDATA%\HyperShot\config.json
```

Ele guarda, entre outras preferências, a pasta sugerida para salvar, formato padrão, atalhos, cor e espessura do traço. A configuração é criada automaticamente na primeira execução.

## Testar antes de alterar

Para conferir se o projeto continua saudável na sua máquina:

```powershell
cargo test --all-targets
```

Para executar uma demonstração que captura a tela primária, aplica algumas anotações e grava `hypershot_demo.png` na pasta atual:

```powershell
cargo run --release -- --demo
```

## Para quem quer contribuir

Ajuda é bem-vinda — inclusive de quem está começando. Alguns bons pontos de entrada são melhorar a documentação, testar comportamentos no Windows, relatar falhas de captura, escrever testes e ajudar a levar a interface nativa para Linux e macOS.

Antes de abrir uma contribuição:

1. Leia o [guia de contribuição](CONTRIBUTING.md).
2. Crie uma branch para uma mudança pequena e clara.
3. Rode `cargo test --all-targets`.
4. Explique no pull request o que mudou e como você testou.

Por favor, não inclua senhas, tokens, chaves de API ou capturas contendo dados pessoais em issues, commits ou pull requests.

## Estrutura do projeto

| Pasta ou arquivo | Para que serve |
| --- | --- |
| `src/core/` | Captura, filtros de imagem, anotações, área de transferência, configuração e união de quadros. |
| `src/ui/` | Sobreposição de seleção, barras de ferramentas, janela fixada e integração com a bandeja do Windows. |
| `assets/` | Ícones do aplicativo. |
| `installer/` | Scripts e arquivos de empacotamento para Windows, Linux e macOS. |
| `.github/workflows/` | Automação de testes e de builds de distribuição. |

## Licença

HyperShot é distribuído sob **MIT ou Apache License 2.0**, a seu critério. Leia [LICENSE-MIT](LICENSE-MIT) e [LICENSE-APACHE](LICENSE-APACHE) para os textos completos.

## English summary

HyperShot is an open-source Rust screenshot tool. Its complete end-user experience is currently implemented for **Windows**: system-tray integration, global hotkeys, a region-selection overlay, annotations, pixelation, clipboard/file export, desktop pinning and scroll stitching. Linux and macOS packaging scaffolding exists, but their native UI is not ready for end users yet.

To build it on Windows:

```powershell
git clone https://github.com/danielccustodio/hypershot.git
cd hypershot
cargo run --release
```

See [CONTRIBUTING.md](CONTRIBUTING.md) to help improve the project.
