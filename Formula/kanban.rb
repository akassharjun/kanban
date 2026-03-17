class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.2.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.2.0/kanban-macos-aarch64.tar.gz"
      sha256 "27eb71dee2dd74829c71b9b506cc2b0f11d2b2f3a6e6b3095fbe840b0dc8267d"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.2.0/kanban-macos-x86_64.tar.gz"
      sha256 "0544548182ee5b8d1892a339e7f7ff24075bd3bbc6c568877366f0bc46352b40"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.2.0/kanban-linux-x86_64.tar.gz"
    sha256 "463e5f093c4d0d474e98a88fe24f653a9ccfc01c08f7d19940aa90e0cc8499da"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
