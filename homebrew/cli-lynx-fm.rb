class CliLynxFm < Formula
  desc "Command-line interface for Lynx.fm music streaming service"
  homepage "https://github.com/dylantarre/lynx-cli"
  url "https://github.com/dylantarre/lynx-cli/archive/refs/tags/v0.1.1.tar.gz"
  sha256 "c741a25fde3ee228fff547e6dbd59756c93a40973f7e23ce2e4efba3e59191a3"
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