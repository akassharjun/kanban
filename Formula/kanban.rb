class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.8.3"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.3/kanban-macos-aarch64.tar.gz"
      sha256 "6a478baf80813af17458f81464b4ead7ca3edb3cedd1d81b14c0de1290a82a40"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.3/kanban-macos-x86_64.tar.gz"
      sha256 "4ec701e8d184958ffd60663b69ca49ac90232e17c3649c41a8fe5fc837fcbc84"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.8.3/kanban-linux-x86_64.tar.gz"
    sha256 "a44eb5fa1f18decb75479a0d41fd53db0aadc644f5ce57a5512f303ff10dedd7"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
