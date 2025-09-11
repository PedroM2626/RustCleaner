# Intelligent Disk Cleaner

Uma aplicação desktop em Rust para limpeza inteligente de disco, desenvolvida com uma interface gráfica moderna usando egui/eframe.

## 📋 Funcionalidades

- **Análise de Disco**: Varredura completa de diretórios para identificar arquivos
- **Categorização Inteligente**: Classificação automática de arquivos por tipo (logs, temporários, cache, etc.)
- **Detecção de Duplicatas**: Identificação de arquivos duplicados baseada em hash
- **Interface Gráfica Intuitiva**: Interface moderna e responsiva construída com egui
- **Visualização de Progresso**: Barras de progresso em tempo real durante as operações
- **Configurações Personalizáveis**: Opções para excluir diretórios e tipos de arquivo específicos
- **Visualização de Resultados**: Exibição detalhada dos arquivos encontrados antes da limpeza
- **Segurança**: Confirmação antes de deletar arquivos importantes

## 🚀 Instalação

### Pré-requisitos

- Rust 1.70 ou superior
- Cargo

### Passos de Instalação

1. Clone o repositório:
```bash
git clone https://github.com/seu-usuario/intelligent-disk-cleaner.git
cd intelligent-disk-cleaner
```

2. Compile o projeto:
```bash
cargo build --release
```

3. Execute a aplicação:
```bash
cargo run --release
```

## 📦 Dependências

- `egui` 0.28 - Framework de interface gráfica
- `eframe` 0.28 - Backend nativo para egui
- `walkdir` - Para navegação em diretórios
- `rayon` - Para processamento paralelo
- `blake3` - Para geração de hash de arquivos
- `serde` - Para serialização/deserialização
- `dirs` - Para acesso a diretórios do sistema

## 🎯 Como Usar

1. **Iniciar Análise**: Clique no botão "Start Scan" para iniciar a varredura
2. **Selecionar Diretório**: Escolha o diretório que deseja analisar
3. **Aguardar Conclusão**: Acompanhe o progresso na barra de status
4. **Revisar Resultados**: Visualize os arquivos encontrados por categoria
5. **Selecionar para Limpeza**: Marque os arquivos que deseja remover
6. **Executar Limpeza**: Confirme e execute a limpeza

## 🛠️ Desenvolvimento

### Estrutura do Projeto

```
src/
├── app.rs           # Lógica principal da interface
├── main.rs          # Ponto de entrada
├── scanner.rs       # Módulo de varredura de arquivos
├── duplicate_finder.rs # Módulo de detecção de duplicatas
├── cleaner.rs       # Módulo de limpeza de arquivos
├── file_category.rs # Categorização de arquivos
├── progress.rs      # Gerenciamento de progresso
└── config.rs        # Configurações da aplicação
```

### Executando em Modo Desenvolvimento

```bash
cargo run
```

### Executando Testes

```bash
cargo test
```

### Formatando Código

```bash
cargo fmt
```

### Análise de Código

```bash
cargo clippy
```

## ⚙️ Configuração

A aplicação utiliza um arquivo de configuração `config.toml` localizado em:
- Windows: `%APPDATA%\intelligent-disk-cleaner\config.toml`
- Linux/macOS: `~/.config/intelligent-disk-cleaner/config.toml`

### Exemplo de Configuração

```toml
[scan]
exclude_dirs = ["/proc", "/sys", "/dev"]
max_file_size = 1073741824  # 1GB

[cleanup]
safe_mode = true
backup_before_delete = false
```

## 🔧 Variáveis de Ambiente

Crie um arquivo `.env` na raiz do projeto com:

```bash
RUST_LOG=debug
RUST_BACKTRACE=1
```

## 🐛 Solução de Problemas

### Erro de Compilação

Se encontrar erros de compilação relacionados ao eframe:

1. Verifique se está usando a versão correta do Rust
2. Atualize as dependências:
```bash
cargo update
```

### Erro de Permissão

Em sistemas Unix/Linux, pode ser necessário permissões de administrador para acessar alguns diretórios.

## 🤝 Contribuindo

1. Faça um fork do projeto
2. Crie uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abra um Pull Request

## 📄 Licença

Este projeto está sob a licença MIT. Veja o arquivo [LICENSE](LICENSE) para mais detalhes.

## 🙋‍♂️ Suporte

Se você encontrar problemas ou tiver dúvidas:

1. Verifique as [Issues](https://github.com/seu-usuario/intelligent-disk-cleaner/issues)
2. Crie uma nova Issue descrevendo o problema
3. Entre em contato através do email: seu-email@example.com

## 🔄 Changelog

### v1.0.0
- Versão inicial com funcionalidades básicas de limpeza
- Interface gráfica com egui
- Detecção de arquivos duplicados
- Categorização de arquivos