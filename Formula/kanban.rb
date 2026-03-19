class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.6.2"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.6.2/kanban-macos-aarch64.tar.gz"
      sha256 "2dde030af277ba46876e6b02989ea37b7fb16f49c8f2ddd803fd0e490fd58e0e"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.6.2/kanban-macos-x86_64.tar.gz"
      sha256 "bc47196fc65d91f5dfccf6a3508978ce358257e0fa98d511949ac974fd53bc2a"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.6.2/kanban-linux-x86_64.tar.gz"
    sha256 "329ffb7824d0b9c2cf9870476063ce1f1cc770350f8309883ada0d358d4d39bb"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
