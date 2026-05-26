# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_barto_cli_global_optspecs
	string join \n v/verbose q/quiet e/enable-std-output c/config-absolute-path= t/tracing-absolute-path= h/help V/version
end

function __fish_barto_cli_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_barto_cli_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_barto_cli_using_subcommand
	set -l cmd (__fish_barto_cli_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c barto-cli -n "__fish_barto_cli_needs_command" -s c -l config-absolute-path -d 'Specify the absolute path to the config file' -r
complete -c barto-cli -n "__fish_barto_cli_needs_command" -s t -l tracing-absolute-path -d 'Specify the absolute path to the tracing output file' -r
complete -c barto-cli -n "__fish_barto_cli_needs_command" -s v -l verbose -d 'Turn up logging verbosity (multiple will turn it up more)'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -s q -l quiet -d 'Turn down logging verbosity (multiple will turn it down more)'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -s e -l enable-std-output -d 'Enable logging to stdout/stderr'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -s h -l help -d 'Print help'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -s V -l version -d 'Print version'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -f -a "info" -d 'Display the bartos version information'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -f -a "updates" -d 'Check for recent updates on a bartoc client'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -f -a "cleanup" -d 'Perform cleanup of old database entries'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -f -a "clients" -d 'List the currently connected clients'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -f -a "query" -d 'Run a query on bartos'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -f -a "list" -d 'List the output for the given command'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -f -a "failed" -d 'List the jobs that failed'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -f -a "cmd" -d 'Display output for the given command name across all clients'
complete -c barto-cli -n "__fish_barto_cli_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand info" -s j -l json -d 'Output the information in JSON format'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand info" -s h -l help -d 'Print help'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand updates" -s n -l name -d 'The name of the bartoc client to check for recent updates' -r
complete -c barto-cli -n "__fish_barto_cli_using_subcommand updates" -s u -l update-kind -d 'The kind of updates to check for' -r
complete -c barto-cli -n "__fish_barto_cli_using_subcommand updates" -s h -l help -d 'Print help'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand cleanup" -s h -l help -d 'Print help'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand clients" -s h -l help -d 'Print help'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand query" -s q -l query -d 'The query to run on bartos' -r
complete -c barto-cli -n "__fish_barto_cli_using_subcommand query" -s h -l help -d 'Print help'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand list" -s n -l name -d 'The name of the bartoc client to check for recent updates' -r
complete -c barto-cli -n "__fish_barto_cli_using_subcommand list" -s c -l cmd-name-opt -d 'The name of the command to list the output for' -r
complete -c barto-cli -n "__fish_barto_cli_using_subcommand list" -s h -l help -d 'Print help'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand failed" -s h -l help -d 'Print help'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand cmd" -s h -l help -d 'Print help'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand help; and not __fish_seen_subcommand_from info updates cleanup clients query list failed cmd help" -f -a "info" -d 'Display the bartos version information'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand help; and not __fish_seen_subcommand_from info updates cleanup clients query list failed cmd help" -f -a "updates" -d 'Check for recent updates on a bartoc client'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand help; and not __fish_seen_subcommand_from info updates cleanup clients query list failed cmd help" -f -a "cleanup" -d 'Perform cleanup of old database entries'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand help; and not __fish_seen_subcommand_from info updates cleanup clients query list failed cmd help" -f -a "clients" -d 'List the currently connected clients'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand help; and not __fish_seen_subcommand_from info updates cleanup clients query list failed cmd help" -f -a "query" -d 'Run a query on bartos'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand help; and not __fish_seen_subcommand_from info updates cleanup clients query list failed cmd help" -f -a "list" -d 'List the output for the given command'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand help; and not __fish_seen_subcommand_from info updates cleanup clients query list failed cmd help" -f -a "failed" -d 'List the jobs that failed'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand help; and not __fish_seen_subcommand_from info updates cleanup clients query list failed cmd help" -f -a "cmd" -d 'Display output for the given command name across all clients'
complete -c barto-cli -n "__fish_barto_cli_using_subcommand help; and not __fish_seen_subcommand_from info updates cleanup clients query list failed cmd help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
