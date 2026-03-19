class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.8.5"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.5/kanban-macos-aarch64.tar.gz"
      sha256 "35434bfa5032740fbaa6a2e976048edccc8c54c15d1461c0ba0424eb95d513a9"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.5/kanban-macos-x86_64.tar.gz"
      sha256 "c220765b8c90f53d732e6c8c6f05ca509b5702c016f09cf732c42947291b8ea2"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.8.5/kanban-linux-x86_64.tar.gz"
    sha256 "9c3fea4ed9ba7277cc6c2a0485eb23c14fdaadcc8551c21133e90d129cd03500"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
