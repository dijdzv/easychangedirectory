apt-get update
apt-get install -y \
  curl \
  git \
  sudo \
  zsh \
  fish \
  build-essential \
  wget apt-transport-https software-properties-common

wget -q "https://packages.microsoft.com/config/ubuntu/$(lsb_release -rs)/packages-microsoft-prod.deb"
sudo dpkg -i packages-microsoft-prod.deb
apt-get update
apt-get install -y powershell

curl https://sh.rustup.rs -sSf | sh -s -- -y
export PATH=/root/.cargo/bin:$PATH
# rustup component add rustfmt
# rustup component add clippy

# cargo install cargo-expand
# cargo install cargo-edit
cargo install --locked easychangedirectory
echo 'eval "$(easychangedirectory --init bash)"' >> ~/.bashrc
# echo 'Invoke-Expression (& { (easychangedirectory --init powershell | Out-String) } )' >> ~/.config/powershell/Microsoft.PowerShell_profile.ps1
# echo 'easychangedirectory --init fish | source' >> ~/.config/fish/config.fish
echo 'eval "$(easychangedirectory --init zsh)"' >> ~/.zshrc

cargo install rust-script