class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.9.3"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.9.3/kanban-macos-aarch64.tar.gz"
      sha256 "e52e841f1dcf7f8a31bbc716cc7b1bc970cf65ff4875b9f21e470796d4f6189a"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.9.3/kanban-macos-x86_64.tar.gz"
      sha256 "33ea8d2ff00d333f45e9a95b4d9c6d8c637ed22d64d028d51333e3c0afb28361"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.9.3/kanban-linux-x86_64.tar.gz"
    sha256 "86a2900f97898c3355e26a60b4c8d116e61fba6ff8a80a43acbf7cffd67054c4"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
