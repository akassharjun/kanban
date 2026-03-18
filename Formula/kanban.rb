class Kanban < Formula
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"
  version "0.4.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/akassharjun/kanban/releases/download/v0.4.0/kanban-macos-aarch64.tar.gz"
      sha256 "2aaebce9f985003a8ade7e687cd29a3bac57fe4c403a37de2dd6d80ac25f7652"
    else
      url "https://github.com/akassharjun/kanban/releases/download/v0.4.0/kanban-macos-x86_64.tar.gz"
      sha256 "da206479b53278f2e084e4c747ebcadf518d4b75c2feed624d25b69198dd2718"
    end
  end

  on_linux do
    url "https://github.com/akassharjun/kanban/releases/download/v0.4.0/kanban-linux-x86_64.tar.gz"
    sha256 "030644ca0532997fb9b3cb372b1bb0cb92e5c35d025a91ad8e1e67a557429001"
  end

  def install
    bin.install "kanban"
  end

  test do
    assert_match "Kanban", shell_output("#{bin}/kanban --help")
  end
end
