class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.6.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.6.0/kanban-macos-aarch64.tar.gz"
      sha256 "8fd4839e637f20b8720970834b655bb5bfe47143ce6c20e9eef38c512941c2f2"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.6.0/kanban-macos-x86_64.tar.gz"
      sha256 "2be59620dd0962d192147e5120fc808f631d247dd910a6499c88eae45db09a16"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.6.0/kanban-linux-x86_64.tar.gz"
    sha256 "247cf354ce92d2fc97df3d79fe072c003cfee739599bbd4c4aa8ee5cd129b037"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
