class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.8.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.1/kanban-macos-aarch64.tar.gz"
      sha256 "96d1146f40246d7bbec3bee4ca7c6d9d988e3bee46ebb2a6e54f3388c47bea72"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.1/kanban-macos-x86_64.tar.gz"
      sha256 "566ee85094c9cbacf6033a3b4201375adfda349b13ce3f4a74e3677bc209840a"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.8.1/kanban-linux-x86_64.tar.gz"
    sha256 "1f5dc183626bb3b0c0e6c3dc495542e7d62a376fc1dd10ff930a83633305d019"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
