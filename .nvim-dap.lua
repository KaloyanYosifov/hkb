local dap = require("dap")

dap.configurations.rust = {
    {
        name = "hkb_client",
        type = "lldb",
        request = "launch",
        program = function()
            return vim.fn.getcwd() .. "/target/debug/hkb_client"
        end,
        cwd = "${workspaceFolder}/hkb_client",
        stopOnEntry = false,
    },

    {
        name = "hkb_daemon",
        type = "lldb",
        request = "launch",
        program = function()
            return vim.fn.getcwd() .. "/target/debug/hkb_daemon"
        end,
        cwd = "${workspaceFolder}/hkb_daemon",
        stopOnEntry = false,
    }
}
