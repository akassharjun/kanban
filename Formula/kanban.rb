class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.1.0"
  license "MIT"

  # SHA256 values are updated automatically by CI on each release.
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.1.0/kanban-macos-aarch64.tar.gz"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.1.0/kanban-macos-x86_64.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.1.0/kanban-linux-x86_64.tar.gz"
    sha256 "PLACEHOLDER"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
