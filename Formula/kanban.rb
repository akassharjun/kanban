class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.8.2"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.2/kanban-macos-aarch64.tar.gz"
      sha256 "669fdc3738ba27318f0b2bb241a43a8f18a39a91a4258e5e0f58cd2e4959031a"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.2/kanban-macos-x86_64.tar.gz"
      sha256 "b18c85508b0c8a06e9ab033095f598f48e476df6a2a7912ddc52b34ab3b7968b"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.8.2/kanban-linux-x86_64.tar.gz"
    sha256 "8243a1b47606b060a808652a7e62c3b8844dcf678ba0d08ecd575c9baa3eb63b"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
