cask "kanban" do
  version "0.6.2"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "a95480a1c1a1f346aa462c07448cbe48b52707ebdd9e934c4b77dac9357205d2"
  else
    sha256 "4a72675123042d5b14a2462a7ecf5c86c0fef4caffdfffd38ac527dc97ec6a45"
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
