class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.8.7"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.7/kanban-macos-aarch64.tar.gz"
      sha256 "bd3e3afd32fe6fbef94d5111f1ca423a89b8b8d164c51cfa4a19559271394491"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.7/kanban-macos-x86_64.tar.gz"
      sha256 "aafc65950724ec57ed67170c4fbc20c380b253e9b6da32c39eeddeed7fba5ae0"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.8.7/kanban-linux-x86_64.tar.gz"
    sha256 "ef870a2c3e049a09be8190a951811fd51fcf8eb2d208cffc8088659f99996ace"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
