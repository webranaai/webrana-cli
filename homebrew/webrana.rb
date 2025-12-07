# ============================================
# WEBRANA AI - Homebrew Formula
# Created by: ATLAS (Team Beta)
# ============================================
# 
# Installation:
#   brew tap webrana/tap
#   brew install webrana
#
# Update this file when releasing new versions.
# The release workflow will auto-update the SHA256.
# ============================================

class Webrana < Formula
  desc "Autonomous CLI Agent with MCP support and multi-model AI"
  homepage "https://github.com/webrana/webrana-ai"
  version "0.3.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/webrana/webrana-ai/releases/download/v#{version}/webrana-macos-aarch64.tar.gz"
      sha256 "PLACEHOLDER_ARM64_SHA256"
    else
      url "https://github.com/webrana/webrana-ai/releases/download/v#{version}/webrana-macos-x86_64.tar.gz"
      sha256 "PLACEHOLDER_X86_64_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/webrana/webrana-ai/releases/download/v#{version}/webrana-linux-aarch64.tar.gz"
      sha256 "PLACEHOLDER_LINUX_ARM64_SHA256"
    else
      url "https://github.com/webrana/webrana-ai/releases/download/v#{version}/webrana-linux-x86_64.tar.gz"
      sha256 "PLACEHOLDER_LINUX_X86_64_SHA256"
    end
  end

  def install
    bin.install "webrana"
  end

  def caveats
    <<~EOS
      Webrana AI has been installed!

      To get started, you'll need an API key:
        export ANTHROPIC_API_KEY=your-key-here
        # or
        export OPENAI_API_KEY=your-key-here

      Quick start:
        webrana chat "Hello, I'm ready to code!"
        webrana run "Create a hello world in Python"

      Documentation: https://github.com/webrana/webrana-ai
    EOS
  end

  test do
    assert_match "webrana", shell_output("#{bin}/webrana --version")
  end
end
