cask "kanban" do
  version "0.9.1"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "039c60dd94227adee1c51649801b7a7f1419b935c3623b2467bfc155d292e80a"
  else
    sha256 "23ac507ebcacab61a551a033fb5d999c4e10b612c40d113928417f785bbf854d"
  end

  name "Kanban"
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"

  app "Kanban.app"

  zap trash: [
    "~/.kanban",
    "~/Library/Caches/com.kanban.desktop",
    "~/Library/Preferences/com.kanban.desktop.plist",
    "~/Library/WebKit/com.kanban.desktop",
  ]
end
