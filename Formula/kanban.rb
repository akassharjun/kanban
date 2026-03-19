class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.8.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.0/kanban-macos-aarch64.tar.gz"
      sha256 "7876355c63068c00fd21ceb253d1e074b915b18ad75306cdf0e482ea1f834629"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.0/kanban-macos-x86_64.tar.gz"
      sha256 "404763954626a5f6f2db8a395670bbf23323f1fe0fc4e1180f9dc5d4a03d711f"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.8.0/kanban-linux-x86_64.tar.gz"
    sha256 "07103d4d05c66430d437f31d6b39112790e3fc6e11f6db5c0a73f683068bc3d4"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
