-- Map `.pp` to the `prepoly` filetype. `vim.filetype.add` takes precedence over
-- the bundled ftdetect that otherwise maps `*.pp` to Puppet. Sourced at startup
-- (plugin managers source `ftdetect/` eagerly), so the first `.pp` buffer is
-- recognised before `vim.lsp.enable("prepoly")` needs the filetype.
vim.filetype.add({ extension = { pp = "prepoly" } })
