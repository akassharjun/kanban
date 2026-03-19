cask "kanban" do
  version "0.8.6"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "ce5c903acfbeee7bfe6b5454d32ad907397b200cd674ec4313d2359b4bd08b52"
  else
    sha256 "c14350a45485a7bf20b0bccfece74b282f0d137e34614147ebee41470725bed6"
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
