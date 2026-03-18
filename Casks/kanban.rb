cask "kanban" do
  version "0.2.0"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "b924eb3da1d0a16a6b1a91a75cbad440934d6681cd9363ba93820d8dbec1755a"
  else
    sha256 "b44dd89f5f2105578b82a07db0518e757ccb7b2024e1c55f077c00e3595880a1"
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
