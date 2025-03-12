class CliLynxFm < Formula
  desc "Command-line interface for Lynx.fm music streaming service"
  homepage "https://github.com/yourusername/cli-lynx-fm"
  url "https://github.com/yourusername/cli-lynx-fm/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
    # Install shell completions
    generate_completions_from_executable(bin/"lynx-fm", "completions")
  end

  test do
    assert_match "Lynx.fm CLI", shell_output("#{bin}/lynx-fm --help")
  end
end 