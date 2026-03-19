class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.8.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.0/kanban-macos-aarch64.tar.gz"
      sha256 "011aa5d935dfed62369b7f7557f11681312189d17f4f9816e1c7736f84630b3d"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.0/kanban-macos-x86_64.tar.gz"
      sha256 "f1c578441591548e041c170c5a2bb9439729eb221e847a5094c18ac0502cbc35"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.8.0/kanban-linux-x86_64.tar.gz"
    sha256 "9272577d7994b247319cc1896217c6647515e78bdb41e4561f8f928f911d38b4"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
