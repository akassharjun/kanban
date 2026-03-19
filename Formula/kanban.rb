class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.7.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.7.0/kanban-macos-aarch64.tar.gz"
      sha256 "a54ca0cd74281f0ff8a62aec5b35870fa40cc4048555b4ec0a86cb9da0059212"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.7.0/kanban-macos-x86_64.tar.gz"
      sha256 "9df2881a7fab4833aa4e5deba5bb5b16d12b8befda22d6a7ef7679a470d45c94"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.7.0/kanban-linux-x86_64.tar.gz"
    sha256 "ad48f6b7a519b943444d17cbcd4a10078ed6d2554a80f5d3f52d1965ef1e375b"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
