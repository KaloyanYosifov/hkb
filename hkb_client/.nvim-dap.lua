local dap = require("dap")
local utils = require("my-config.utils")


dap.adapters.lldb = {
    type = "executable",
    command = utils.command_path("lldb-vscode"),
    name = "lldb"
}

dap.configurations.rust = {
    {
        name = "hkb_client",
        type = "lldb",
        request = "launch",
        program = function()
            return vim.fn.getcwd() .. "/target/debug/hkb_client"
        end,
        cwd = "${workspaceFolder}",
        stopOnEntry = false,
    }
}
