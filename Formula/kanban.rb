class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.8.8"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.8/kanban-macos-aarch64.tar.gz"
      sha256 "86a5f5414ac55fd97ed002e24ec6ac4c2f2ac8949725cafbb13768a187b2cd2a"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.8/kanban-macos-x86_64.tar.gz"
      sha256 "40153cfe9101376027dd57c3342c940a1ef836c92eb8d3af13c752406cbd52f0"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.8.8/kanban-linux-x86_64.tar.gz"
    sha256 "425b66b3ca29ec56fab34ebc9454fd3313c598a3820c050214b9ed94a7fda90e"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
