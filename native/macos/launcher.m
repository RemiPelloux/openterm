#import <Foundation/Foundation.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static BOOL configNeedsUpgrade(NSString *path) {
    NSString *text = [NSString stringWithContentsOfFile:path
                                               encoding:NSUTF8StringEncoding
                                                  error:nil];
    if (!text) {
        return YES;
    }
    return ![text containsString:@"openterm-theme-version = 5"];
}

static NSString *supportDir(void) {
    NSURL *base = [NSFileManager.defaultManager
        URLsForDirectory:NSApplicationSupportDirectory
               inDomains:NSUserDomainMask].firstObject;
    return [base.path stringByAppendingPathComponent:@"OpenTerm"];
}

static void prepareSupport(NSString *resources) {
    NSFileManager *files = NSFileManager.defaultManager;
    NSString *support = supportDir();
    NSString *config = [support stringByAppendingPathComponent:@"config.ghostty"];
    NSString *source = [resources stringByAppendingPathComponent:
                        @"openterm/default-config.ghostty"];
    [files createDirectoryAtPath:support
     withIntermediateDirectories:YES attributes:nil error:nil];
    if (![files fileExistsAtPath:config] || configNeedsUpgrade(config)) {
        if ([files fileExistsAtPath:config]) {
            NSString *backup = [config stringByAppendingString:@".bak"];
            [files removeItemAtPath:backup error:nil];
            [files moveItemAtPath:config toPath:backup error:nil];
        }
        [files copyItemAtPath:source toPath:config error:nil];
    }
    NSString *data = [support stringByAppendingPathComponent:@"data"];
    setenv("OPENTERM_DATA_DIR", data.fileSystemRepresentation, 1);
}

static void prependTools(NSString *resources) {
    NSString *tools = [resources stringByAppendingPathComponent:@"openterm/bin"];
    NSString *existing = NSProcessInfo.processInfo.environment[@"PATH"] ?: @"/usr/bin:/bin";
    NSString *path = [NSString stringWithFormat:@"%@:%@", tools, existing];
    setenv("PATH", path.UTF8String, 1);
}

/* The splash floats above the terminal, so anyone who wants the window
   immediately can set OPENTERM_SPLASH=0 in the bundle's LSEnvironment. */
static BOOL splashDisabled(void) {
    const char *value = getenv("OPENTERM_SPLASH");
    return value != NULL && strcmp(value, "0") == 0;
}

static void startSplash(NSString *resources) {
    if (splashDisabled()) {
        return;
    }
    NSString *path = [resources stringByAppendingPathComponent:@"openterm/openterm-splash"];
    NSTask *task = [NSTask new];
    task.executableURL = [NSURL fileURLWithPath:path];
    [task launchAndReturnError:nil];
}

static void prepareShellEnvironment(NSString *resources) {
    NSString *bundle = [resources stringByAppendingPathComponent:@"openterm"];
    NSString *zdot = [bundle stringByAppendingPathComponent:@"shell/zsh"];
    NSString *home = NSHomeDirectory();
    setenv("OPENTERM_RESOURCES", bundle.fileSystemRepresentation, 1);
    setenv("OPENTERM_ORIGINAL_ZDOTDIR", home.fileSystemRepresentation, 1);
    setenv("ZDOTDIR", zdot.fileSystemRepresentation, 1);
}

/* Builds the one file the engine reads, so the CLI flag and GHOSTTY_CONFIG_PATH
   can never describe different configurations. It pulls in the user's theme by
   reference rather than copying it, so edits to config.ghostty take effect on
   the next launch. Returns the path to pass to the engine. */
static NSString *prepareRuntime(NSString *resources) {
    prepareSupport(resources);
    prepareShellEnvironment(resources);
    NSString *support = supportDir();
    NSString *runtime = [support stringByAppendingPathComponent:@"runtime.ghostty"];
    NSString *config = [support stringByAppendingPathComponent:@"config.ghostty"];
    NSString *shell = [resources stringByAppendingPathComponent:@"openterm/shell/openterm-shell"];
    NSString *text = [NSString stringWithFormat:@"config-file = %@\ncommand = direct:%@\n",
                                                config, shell];
    [text writeToFile:runtime atomically:YES encoding:NSUTF8StringEncoding error:nil];
    setenv("GHOSTTY_CONFIG_PATH", runtime.fileSystemRepresentation, 1);
    return runtime;
}

/* We exec the engine in place, so the launch handshake LaunchServices sent to
   this process image is gone by the time AppKit starts. Reporting "app" would
   make the engine wait for an activation that can never arrive, leaving it with
   no window until a second dock click. "cli" makes it activate itself. */
static void declareLaunchSource(void) {
    setenv("GHOSTTY_MAC_LAUNCH_SOURCE", "cli", 1);
}

static void launchTerminal(NSString *terminal, NSString *runtime) {
    const char *engine = terminal.fileSystemRepresentation;
    /* Ghostty CLI only accepts --key=value (not separate args). execv passes
       argv verbatim, so quoting a value would become part of the path. */
    NSString *configFlag = [NSString stringWithFormat:@"--config-file=%@", runtime];
    char *args[] = {
        (char *)engine,
        (char *)"--config-default-files=false",
        (char *)configFlag.UTF8String,
        NULL,
    };
    execv(engine, args);
}

int main(int argc, char *argv[]) {
    @autoreleasepool {
        NSString *contents = NSBundle.mainBundle.bundlePath;
        contents = [contents stringByAppendingPathComponent:@"Contents"];
        NSString *resources = [contents stringByAppendingPathComponent:@"Resources"];
        NSString *runtime = prepareRuntime(resources);
        prependTools(resources);
        declareLaunchSource();
        startSplash(resources);
        NSString *terminal = [contents stringByAppendingPathComponent:
                              @"MacOS/openterm-terminal"];
        launchTerminal(terminal, runtime);
        NSLog(@"OpenTerm could not start terminal engine: %s", strerror(errno));
        return 127;
    }
    (void)argc;
    (void)argv;
}
