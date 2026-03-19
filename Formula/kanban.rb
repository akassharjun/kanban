class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.8.4"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.4/kanban-macos-aarch64.tar.gz"
      sha256 "b53ed7c3972725415eb4721431687baeb15c81f62f16f41fbb1d3027e5a0f21b"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.8.4/kanban-macos-x86_64.tar.gz"
      sha256 "8ac9d91d8766d946719cc050ec2b32ab9ced227797ef02ff2251876d09d0777f"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.8.4/kanban-linux-x86_64.tar.gz"
    sha256 "dfe2f719be5e1bc775718f4682e44a4f20167c34b53347c4127033e7a951ad92"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
