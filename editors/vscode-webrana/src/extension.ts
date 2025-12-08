import * as vscode from 'vscode';
import * as cp from 'child_process';
import * as path from 'path';

let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel('Webrana');
    
    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('webrana.chat', startChat),
        vscode.commands.registerCommand('webrana.run', runTask),
        vscode.commands.registerCommand('webrana.explain', explainSelection),
        vscode.commands.registerCommand('webrana.fix', fixSelection),
        vscode.commands.registerCommand('webrana.test', generateTests),
        vscode.commands.registerCommand('webrana.scan', scanSecrets)
    );

    outputChannel.appendLine('Webrana extension activated');
}

export function deactivate() {
    outputChannel.dispose();
}

// Get webrana executable path from settings
function getWebranaPath(): string {
    const config = vscode.workspace.getConfiguration('webrana');
    return config.get<string>('executablePath', 'webrana');
}

// Get workspace folder
function getWorkspaceFolder(): string {
    const folders = vscode.workspace.workspaceFolders;
    if (folders && folders.length > 0) {
        return folders[0].uri.fsPath;
    }
    return process.cwd();
}

// Execute webrana command
async function executeWebrana(args: string[]): Promise<string> {
    const webranaPath = getWebranaPath();
    const workdir = getWorkspaceFolder();

    return new Promise((resolve, reject) => {
        const proc = cp.spawn(webranaPath, args, {
            cwd: workdir,
            env: process.env
        });

        let stdout = '';
        let stderr = '';

        proc.stdout.on('data', (data) => {
            stdout += data.toString();
            outputChannel.append(data.toString());
        });

        proc.stderr.on('data', (data) => {
            stderr += data.toString();
            outputChannel.append(data.toString());
        });

        proc.on('close', (code) => {
            if (code === 0) {
                resolve(stdout);
            } else {
                reject(new Error(stderr || `Process exited with code ${code}`));
            }
        });

        proc.on('error', (err) => {
            reject(err);
        });
    });
}

// Start chat session
async function startChat() {
    const message = await vscode.window.showInputBox({
        prompt: 'Enter your message for Webrana',
        placeHolder: 'e.g., Help me refactor this function'
    });

    if (!message) {
        return;
    }

    outputChannel.show();
    outputChannel.appendLine(`\n>>> ${message}\n`);

    try {
        const result = await executeWebrana(['chat', message]);
        outputChannel.appendLine(result);
    } catch (err: any) {
        vscode.window.showErrorMessage(`Webrana error: ${err.message}`);
    }
}

// Run autonomous task
async function runTask() {
    const task = await vscode.window.showInputBox({
        prompt: 'Enter task for Webrana to execute',
        placeHolder: 'e.g., Add error handling to all API calls'
    });

    if (!task) {
        return;
    }

    const config = vscode.workspace.getConfiguration('webrana');
    const maxIterations = config.get<number>('maxIterations', 10);

    outputChannel.show();
    outputChannel.appendLine(`\n>>> Running task: ${task}\n`);

    try {
        const result = await executeWebrana([
            'run', task,
            '--max-iterations', maxIterations.toString()
        ]);
        outputChannel.appendLine(result);
        vscode.window.showInformationMessage('Task completed');
    } catch (err: any) {
        vscode.window.showErrorMessage(`Webrana error: ${err.message}`);
    }
}

// Explain selected code
async function explainSelection() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        return;
    }

    const selection = editor.selection;
    const text = editor.document.getText(selection);

    if (!text) {
        vscode.window.showWarningMessage('Please select some code first');
        return;
    }

    const language = editor.document.languageId;
    const prompt = `Explain this ${language} code:\n\n\`\`\`${language}\n${text}\n\`\`\``;

    outputChannel.show();
    outputChannel.appendLine(`\n>>> Explain selection\n`);

    try {
        const result = await executeWebrana(['chat', prompt]);
        outputChannel.appendLine(result);
    } catch (err: any) {
        vscode.window.showErrorMessage(`Webrana error: ${err.message}`);
    }
}

// Fix selected code
async function fixSelection() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        return;
    }

    const selection = editor.selection;
    const text = editor.document.getText(selection);

    if (!text) {
        vscode.window.showWarningMessage('Please select some code first');
        return;
    }

    const language = editor.document.languageId;
    const prompt = `Fix any issues in this ${language} code and explain the changes:\n\n\`\`\`${language}\n${text}\n\`\`\``;

    outputChannel.show();
    outputChannel.appendLine(`\n>>> Fix selection\n`);

    try {
        const result = await executeWebrana(['chat', prompt]);
        outputChannel.appendLine(result);
    } catch (err: any) {
        vscode.window.showErrorMessage(`Webrana error: ${err.message}`);
    }
}

// Generate tests for current file
async function generateTests() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        return;
    }

    const filePath = editor.document.uri.fsPath;
    const language = editor.document.languageId;
    const fileName = path.basename(filePath);

    const prompt = `Generate comprehensive unit tests for ${fileName}. Use appropriate testing framework for ${language}.`;

    outputChannel.show();
    outputChannel.appendLine(`\n>>> Generate tests for ${fileName}\n`);

    try {
        const result = await executeWebrana(['chat', prompt]);
        outputChannel.appendLine(result);
    } catch (err: any) {
        vscode.window.showErrorMessage(`Webrana error: ${err.message}`);
    }
}

// Scan workspace for secrets
async function scanSecrets() {
    const workdir = getWorkspaceFolder();

    outputChannel.show();
    outputChannel.appendLine(`\n>>> Scanning for secrets in ${workdir}\n`);

    try {
        const result = await executeWebrana(['scan', '--dir', workdir]);
        outputChannel.appendLine(result);

        if (result.includes('No secrets detected')) {
            vscode.window.showInformationMessage('No secrets detected');
        } else {
            vscode.window.showWarningMessage('Secrets detected! Check output for details.');
        }
    } catch (err: any) {
        vscode.window.showErrorMessage(`Webrana error: ${err.message}`);
    }
}
