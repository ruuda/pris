# This Vagrant box is similar to the environment Travis CI provides. It can be
# helpful to diagnose Ubuntu-specific issues. (Fontconfig can be ... difficult.)

Vagrant.configure("2") do |config|
  config.vm.box = "ubuntu/precise64"

  config.vm.provider "virtualbox" do |vb|
    vb.memory = "1024"
  end

  config.vm.provision "shell", inline: <<-SHELL
    apt-get update
    apt-get install -y git fonts-cantarell libcairo2-dev
    curl -sL https://static.rust-lang.org/rustup.sh -o /tmp/rustup.sh
    chmod a+rx /tmp/rustup.sh
  SHELL
end
