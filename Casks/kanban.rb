cask "kanban" do
  version "0.8.7"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "ba72e918fcc12e768fd9928d1bd7d7929df169b737de2100fde3b906c777a0fa"
  else
    sha256 "abb7ab5d8f799f8f37043219298efd28fe28fe88b23a4a5aa96267ae747dff06"
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
