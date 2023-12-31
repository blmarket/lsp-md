Lsp-md
------

Language server implementation for markdown documents.

## Introduction

This repository builds `lsp-md`, which is a specialized implementation of the
Language Server Protocol (LSP) for personal markdown notes.

## Core features

- Vector embedding and its related features allow for:
  - Finding similar notes
  - Searching notes by keyword to find relevant information
- The formatting feature is superior to the `gq` command in Vim as it offers the
  following improvements:
  - Wrapping text to fit within 80 columns
  - Placing URLs at the beginning of the line for easier `gx`
  - Preserving line breaks (double space at the end of the line) when formatting
    text

## Motivation

Wanted to do more with nvim, and LSP looked promising:

* Being a server, it can effectively maintain vector embeddings.
* It is highly customizable as long as it adheres to the LSP protocol.

## Run the server

`cargo run`

Configuring standalone binaries can be challenging due to the presence of
dynamic linked libraries from PyTorch. `LD_LIBRARY_PATH=somewhere_in_deps`

## Integration with nvim

Most basic way to integrate is to use lspconfig.

```lua
local lspconfig = require("lspconfig")
local configs = require("lspconfig.configs")
local util = require("lspconfig.util")

-- My own lsp server for markdown
if not configs.lsp_md then
  configs.lsp_md = {
    default_config = {
      cmd_cwd = "/home/blmarket/proj/lsp-md/target/release",
      cmd = { 'sh', '-c', 'LD_LIBRARY_PATH=. ./lsp-md' }, -- no other stable way to run command
      filetypes = { 'markdown' },
      root_dir = function(fname) -- use same root_dir policy with marksman
        local root_files = { '.marksman.toml' }
        return util.root_pattern(unpack(root_files))(fname) or util.find_git_ancestor(fname)
      end,
      single_file_support = true,
    },
  }
end

lspconfig.lsp_md.setup({})
```

### More functionality

Actually similar doc and keyword search is implemented using
`workspace/executeCommand` which is not usable without custom integration. There
are pages of custom integration integrating those commands with telescope etc.
to make it work. For actual usages see my own dotfiles for references.

