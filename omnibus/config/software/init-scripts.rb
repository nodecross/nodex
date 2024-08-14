name "init-scripts"
# always_build true

ohai = Ohai::System.new
ohai.all_plugins

build do
  if ohai['platform_family'] == 'debian'
    # sysvinit support for debian only for now
    mkdir "/etc/init.d"
    # debian recommends using a different directory for systemd unit files
    etc_dir = "/etc/nodex-agent"
    systemd_directory = "/lib/systemd/system"
    # project.extra_package_file '/etc/init.d/nodex-agent'
    erb source: "systemd.service.erb",
        dest: "#{systemd_directory}/nodex-agent.service",
        mode: 0644,
        vars: { install_dir: install_dir, etc_dir: etc_dir }
  end
end
