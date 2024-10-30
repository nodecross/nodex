name "init-scripts"
ohai = Ohai::System.new
ohai.all_plugins

build do
  if ohai['platform_family'] == 'debian'
    etc_dir = "/etc/nodex-agent"
    systemd_directory = "/lib/systemd/system"
    erb source: "systemd.socket.erb",
      dest: "#{systemd_directory}/nodex-agent.socket",
      mode: 0644,
      vars: { install_dir: install_dir, etc_dir: etc_dir }
    erb source: "systemd.service.erb",
      dest: "#{systemd_directory}/nodex-agent.service",
      mode: 0644,
      vars: { install_dir: install_dir, etc_dir: etc_dir }
    project.extra_package_file "#{systemd_directory}/nodex-agent.service"
  end
end
