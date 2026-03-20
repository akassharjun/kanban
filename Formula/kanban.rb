class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.9.2"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.9.2/kanban-macos-aarch64.tar.gz"
      sha256 "4b2c826b89b80f95d1dc99396fc95a695a67939d922468042389bbdbbf2339b7"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.9.2/kanban-macos-x86_64.tar.gz"
      sha256 "fb5f89bb9ca104b3100e45965491bdfb1de7603fbb780751203ce67a1c05ec5b"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.9.2/kanban-linux-x86_64.tar.gz"
    sha256 "2c9ca9a4bf3891e85413550a98ece15f94d7de5808c049e875f9dbc03055db0b"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
