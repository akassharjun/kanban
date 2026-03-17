class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.1.0/kanban-macos-aarch64.tar.gz"
      sha256 "e9b3c21d8f2be88c3cf19eab0afef2a83e38c244f81ae78fb486bb4b23ce63f0"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.1.0/kanban-macos-x86_64.tar.gz"
      sha256 "67a9ecd0e2812356e8919c8c1b64617da42c94922f0380776f5169ebf6bea11d"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.1.0/kanban-linux-x86_64.tar.gz"
    sha256 "83a3643641793a7713d15df51bad41f5802d311261203990cb8859531f4f73c6"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
