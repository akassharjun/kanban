class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.6.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.6.1/kanban-macos-aarch64.tar.gz"
      sha256 "6c00e3b8b7f45372c7267fafca3c9a59cc9d33bd2c52ac78e0f6aceeaaae7fbc"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.6.1/kanban-macos-x86_64.tar.gz"
      sha256 "b97f2ff06fbadf225b05f643b803fb24e4376ec34ac043ac2d6f6ebe132bfdf7"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.6.1/kanban-linux-x86_64.tar.gz"
    sha256 "cd7c5405a51ab53e7d0a7910701c96cbed63db05add2be3053bc7ab79cae0d86"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
