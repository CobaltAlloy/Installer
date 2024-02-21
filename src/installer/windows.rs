use cfg_if::cfg_if;

/// Exit or do the "press and key to exit" thing on windows
pub fn exit_or_windows(code: i32) {
    cfg_if! {
        if #[cfg(target_os = "windows")] {
           let mut stdin = std::io::stdin();
            
            print!("Press enter to exit.. (code {})", code);
            stdout.flush().unwrap();

            let _ = stdin.read(&mut [0u8]).unwrap();
        }
    }

    std::process::exit(code);
}
