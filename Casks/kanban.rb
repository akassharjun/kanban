cask "kanban" do
  version "0.8.1"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "d18d0d292a7b9379cb2e406f2dee052133e8ca84b0bf37d50300dd6066efbd78"
  else
    sha256 "ee0af9cd68e87e557a592db5b01e33a15a411d95b99d64050a8eb96a0b398bab"
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
