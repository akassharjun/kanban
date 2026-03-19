cask "kanban" do
  version "0.8.0"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "45e48e585fcd62040a538314a819a773a970fc6614cc8bb8b6f35faf0e582125"
  else
    sha256 "7f383e3c9e4f75602a91a9b521571bd150648bd003bad85548a16a32daebd5fd"
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
