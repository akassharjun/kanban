class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.9.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.9.1/kanban-macos-aarch64.tar.gz"
      sha256 "b02017bd55f06b3fb69422414c9c127f97bda29ff9509eba0c995240fdb07d58"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.9.1/kanban-macos-x86_64.tar.gz"
      sha256 "c4341c149978949bd174c084a594e835756fca20bf98c16f2116bda977b5d7bc"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.9.1/kanban-linux-x86_64.tar.gz"
    sha256 "6e55767d8fed3c358e9cbf70ee60f778b66eb72fac68fe97b90062b5c34af41a"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
