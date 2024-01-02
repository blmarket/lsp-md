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
dynamic linked libraries from PyTorch. See below alternatives.

### Alternative: patchelf

`ldd lsp-md` to find missing so files, copy them from deps/...(some pytorch
lib) and patchelf --replace-needed to set them current path.

### Alternative 2: copy so files to /usr/local/lib?

Even though it will make those stale torch lib (currently they're 2.0.0, not
the latest version...)

### Alternative 3: Set `LD_LIBRARY_PATH` env

It will help bin to find those so files.

### Not recommended: Static build everything

The problem is from built binary is looking for dynamic lib which is not
installed in the system, so static link can be an option to solve the problem.
Just following problems:

* Impossible to decide everything at the build time:  
  we don't know the target system has cuda or other BLAS libs.
* Headache building torch by yourself:  
  You want to just build a language server, not the underlying library.

Overall it does not worth trying.

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

