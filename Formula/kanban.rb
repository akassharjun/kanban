class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.9.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.9.0/kanban-macos-aarch64.tar.gz"
      sha256 "ab168a796e55a98cfec0cb1d9e753880bcd582dd89ec1939282217cd49279696"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.9.0/kanban-macos-x86_64.tar.gz"
      sha256 "a5de91c27d9290e88b114a4d20ee2170aec698c1554cb8f5011af2f641ecf9f0"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.9.0/kanban-linux-x86_64.tar.gz"
    sha256 "a5f0be030301b541a2b0d3d56a4cdc774e2150a9c27528ed042770f20fb66312"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
