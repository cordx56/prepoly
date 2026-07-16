-- Map `.cz` to the `brass` filetype. Sourced at startup (plugin managers source
-- `ftdetect/` eagerly), so the first `.cz` buffer is recognised before
-- `vim.lsp.enable("brass")` needs the filetype.
vim.filetype.add({ extension = { cz = "brass" } })
