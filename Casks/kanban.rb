cask "kanban" do
  version "0.9.3"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "17140a24b5a275b2393da07a4b6fc5d5c1d88f0aa82c73d4f3339c7cae5554b3"
  else
    sha256 "65d6bf925429e71100eb46ab10fe41a492282aad2a39104df584b411112dfa7b"
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
