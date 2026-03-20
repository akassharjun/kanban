cask "kanban" do
  version "0.9.2"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "0ce205541416004b2d06a95ee1729a3f5ed917b631953b059cd97438fb83f6f7"
  else
    sha256 "e8d2d9ff7237d2433fac77fb45f803ba6dbc7e9f1103bc6d2c117a25d42507b2"
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
