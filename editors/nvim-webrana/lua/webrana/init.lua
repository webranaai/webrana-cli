-- Webrana CLI Neovim Plugin
-- AI-powered autonomous coding assistant

local M = {}

-- Default configuration
M.config = {
    executable = "webrana",
    auto_mode = false,
    max_iterations = 10,
}

-- Setup function
function M.setup(opts)
    M.config = vim.tbl_deep_extend("force", M.config, opts or {})
    
    -- Create commands
    vim.api.nvim_create_user_command("WebranaChat", function(args)
        M.chat(args.args)
    end, { nargs = "?" })
    
    vim.api.nvim_create_user_command("WebranaRun", function(args)
        M.run(args.args)
    end, { nargs = 1 })
    
    vim.api.nvim_create_user_command("WebranaExplain", function()
        M.explain()
    end, { range = true })
    
    vim.api.nvim_create_user_command("WebranaFix", function()
        M.fix()
    end, { range = true })
    
    vim.api.nvim_create_user_command("WebranaTest", function()
        M.generate_tests()
    end, {})
    
    vim.api.nvim_create_user_command("WebranaScan", function()
        M.scan()
    end, {})
    
    -- Set up keymaps
    vim.keymap.set("n", "<leader>wc", ":WebranaChat<CR>", { desc = "Webrana Chat" })
    vim.keymap.set("v", "<leader>we", ":WebranaExplain<CR>", { desc = "Webrana Explain" })
    vim.keymap.set("v", "<leader>wf", ":WebranaFix<CR>", { desc = "Webrana Fix" })
    vim.keymap.set("n", "<leader>wt", ":WebranaTest<CR>", { desc = "Webrana Test" })
    vim.keymap.set("n", "<leader>ws", ":WebranaScan<CR>", { desc = "Webrana Scan" })
end

-- Execute webrana command
function M.execute(args, callback)
    local cmd = M.config.executable .. " " .. table.concat(args, " ")
    local output = {}
    
    vim.fn.jobstart(cmd, {
        stdout_buffered = true,
        stderr_buffered = true,
        on_stdout = function(_, data)
            if data then
                for _, line in ipairs(data) do
                    if line ~= "" then
                        table.insert(output, line)
                    end
                end
            end
        end,
        on_stderr = function(_, data)
            if data then
                for _, line in ipairs(data) do
                    if line ~= "" then
                        table.insert(output, line)
                    end
                end
            end
        end,
        on_exit = function(_, code)
            if callback then
                callback(output, code)
            end
        end,
    })
end

-- Start chat
function M.chat(message)
    if not message or message == "" then
        message = vim.fn.input("Webrana> ")
    end
    
    if message == "" then
        return
    end
    
    M.show_output("Webrana Chat")
    M.append_output(">>> " .. message)
    M.append_output("")
    
    M.execute({ "chat", vim.fn.shellescape(message) }, function(output)
        for _, line in ipairs(output) do
            M.append_output(line)
        end
    end)
end

-- Run autonomous task
function M.run(task)
    if not task or task == "" then
        task = vim.fn.input("Task> ")
    end
    
    if task == "" then
        return
    end
    
    M.show_output("Webrana Run")
    M.append_output(">>> Running: " .. task)
    M.append_output("")
    
    local args = {
        "run",
        vim.fn.shellescape(task),
        "--max-iterations",
        tostring(M.config.max_iterations)
    }
    
    M.execute(args, function(output)
        for _, line in ipairs(output) do
            M.append_output(line)
        end
    end)
end

-- Explain selected code
function M.explain()
    local lines = M.get_visual_selection()
    if #lines == 0 then
        vim.notify("No selection", vim.log.levels.WARN)
        return
    end
    
    local code = table.concat(lines, "\n")
    local ft = vim.bo.filetype
    local prompt = string.format("Explain this %s code:\n\n```%s\n%s\n```", ft, ft, code)
    
    M.show_output("Webrana Explain")
    M.append_output(">>> Explaining selection")
    M.append_output("")
    
    M.execute({ "chat", vim.fn.shellescape(prompt) }, function(output)
        for _, line in ipairs(output) do
            M.append_output(line)
        end
    end)
end

-- Fix selected code
function M.fix()
    local lines = M.get_visual_selection()
    if #lines == 0 then
        vim.notify("No selection", vim.log.levels.WARN)
        return
    end
    
    local code = table.concat(lines, "\n")
    local ft = vim.bo.filetype
    local prompt = string.format("Fix issues in this %s code:\n\n```%s\n%s\n```", ft, ft, code)
    
    M.show_output("Webrana Fix")
    M.append_output(">>> Fixing selection")
    M.append_output("")
    
    M.execute({ "chat", vim.fn.shellescape(prompt) }, function(output)
        for _, line in ipairs(output) do
            M.append_output(line)
        end
    end)
end

-- Generate tests
function M.generate_tests()
    local filename = vim.fn.expand("%:t")
    local ft = vim.bo.filetype
    local prompt = string.format("Generate unit tests for %s using appropriate %s testing framework", filename, ft)
    
    M.show_output("Webrana Test")
    M.append_output(">>> Generating tests for " .. filename)
    M.append_output("")
    
    M.execute({ "chat", vim.fn.shellescape(prompt) }, function(output)
        for _, line in ipairs(output) do
            M.append_output(line)
        end
    end)
end

-- Scan for secrets
function M.scan()
    local cwd = vim.fn.getcwd()
    
    M.show_output("Webrana Scan")
    M.append_output(">>> Scanning " .. cwd)
    M.append_output("")
    
    M.execute({ "scan", "--dir", cwd }, function(output)
        for _, line in ipairs(output) do
            M.append_output(line)
        end
    end)
end

-- Get visual selection
function M.get_visual_selection()
    local start_pos = vim.fn.getpos("'<")
    local end_pos = vim.fn.getpos("'>")
    local lines = vim.fn.getline(start_pos[2], end_pos[2])
    return lines
end

-- Output buffer management
local output_buf = nil
local output_win = nil

function M.show_output(title)
    if output_buf and vim.api.nvim_buf_is_valid(output_buf) then
        vim.api.nvim_buf_set_lines(output_buf, 0, -1, false, {})
    else
        output_buf = vim.api.nvim_create_buf(false, true)
        vim.api.nvim_buf_set_option(output_buf, "buftype", "nofile")
        vim.api.nvim_buf_set_option(output_buf, "filetype", "markdown")
    end
    
    if not output_win or not vim.api.nvim_win_is_valid(output_win) then
        vim.cmd("botright vsplit")
        output_win = vim.api.nvim_get_current_win()
        vim.api.nvim_win_set_buf(output_win, output_buf)
        vim.api.nvim_win_set_width(output_win, 60)
    end
    
    vim.api.nvim_buf_set_name(output_buf, title)
end

function M.append_output(line)
    if output_buf and vim.api.nvim_buf_is_valid(output_buf) then
        local lines = vim.api.nvim_buf_get_lines(output_buf, 0, -1, false)
        table.insert(lines, line)
        vim.api.nvim_buf_set_lines(output_buf, 0, -1, false, lines)
    end
end

return M
