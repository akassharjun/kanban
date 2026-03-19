class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.8.6"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.6/kanban-macos-aarch64.tar.gz"
      sha256 "ecc5510b571f67e72577e5a4b20ab625223b870b036fc3f7f5285345ff5352d8"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.6/kanban-macos-x86_64.tar.gz"
      sha256 "d968f4dc79dea14f883b5e12d111ac1ff4078a771670f35d5a47ec9e5534260a"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.8.6/kanban-linux-x86_64.tar.gz"
    sha256 "02452e9eae0ddfc57a5b1083bebc307129c4504cdfbc084b0ef40e6608375f0b"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
