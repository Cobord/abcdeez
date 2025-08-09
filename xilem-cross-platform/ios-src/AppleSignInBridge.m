//
//  AppleSignInBridge.m
//  Graph Learning System
//
//  iOS Bridge Implementation for Apple Sign In with AuthenticationServices
//

#import "AppleSignInBridge.h"
#import <UIKit/UIKit.h>

@implementation AppleSignInBridge

+ (instancetype)sharedInstance {
    static AppleSignInBridge *sharedInstance = nil;
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        sharedInstance = [[self alloc] init];
    });
    return sharedInstance;
}

- (void)startAppleSignIn {
    if (@available(iOS 13.0, *)) {
        ASAuthorizationAppleIDProvider *appleIDProvider = [[ASAuthorizationAppleIDProvider alloc] init];
        ASAuthorizationAppleIDRequest *request = [appleIDProvider createRequest];
        request.requestedScopes = @[ASAuthorizationScopeFullName, ASAuthorizationScopeEmail];
        
        ASAuthorizationController *authorizationController = [[ASAuthorizationController alloc] initWithAuthorizationRequests:@[request]];
        authorizationController.delegate = self;
        authorizationController.presentationContextProvider = self;
        [authorizationController performRequests];
    } else {
        // Fallback for earlier iOS versions
        NSDictionary *errorInfo = @{
            @"error": @"Apple Sign In requires iOS 13.0 or later"
        };
        NSData *jsonData = [NSJSONSerialization dataWithJSONObject:errorInfo options:0 error:nil];
        NSString *jsonString = [[NSString alloc] initWithData:jsonData encoding:NSUTF8StringEncoding];
        
        handle_apple_sign_in_callback([jsonString UTF8String], false);
    }
}

#pragma mark - ASAuthorizationControllerDelegate

- (void)authorizationController:(ASAuthorizationController *)controller didCompleteWithAuthorization:(ASAuthorization *)authorization API_AVAILABLE(ios(13.0)) {
    if ([authorization.credential isKindOfClass:[ASAuthorizationAppleIDCredential class]]) {
        ASAuthorizationAppleIDCredential *appleIDCredential = authorization.credential;
        
        // Extract the identity token
        NSData *identityTokenData = appleIDCredential.identityToken;
        NSString *identityToken = nil;
        if (identityTokenData) {
            identityToken = [[NSString alloc] initWithData:identityTokenData encoding:NSUTF8StringEncoding];
        }
        
        // Extract the authorization code
        NSData *authorizationCodeData = appleIDCredential.authorizationCode;
        NSString *authorizationCode = nil;
        if (authorizationCodeData) {
            authorizationCode = [[NSString alloc] initWithData:authorizationCodeData encoding:NSUTF8StringEncoding];
        }
        
        // Extract user information
        NSPersonNameComponents *fullName = appleIDCredential.fullName;
        NSString *email = appleIDCredential.email;
        NSString *userIdentifier = appleIDCredential.user;
        
        // Create the response data structure
        NSMutableDictionary *userData = [[NSMutableDictionary alloc] init];
        
        if (userIdentifier) {
            userData[@"user_id"] = userIdentifier;
        }
        
        if (identityToken) {
            userData[@"identity_token"] = identityToken;
        }
        
        if (authorizationCode) {
            userData[@"authorization_code"] = authorizationCode;
        }
        
        if (email) {
            userData[@"email"] = email;
        }
        
        if (fullName) {
            NSMutableDictionary *nameDict = [[NSMutableDictionary alloc] init];
            if (fullName.givenName) {
                nameDict[@"given_name"] = fullName.givenName;
            }
            if (fullName.familyName) {
                nameDict[@"family_name"] = fullName.familyName;
            }
            if (nameDict.count > 0) {
                userData[@"full_name"] = nameDict;
            }
        }
        
        // Add real user status
        userData[@"real_user_status"] = @(appleIDCredential.realUserStatus);
        
        // Convert to JSON
        NSError *error;
        NSData *jsonData = [NSJSONSerialization dataWithJSONObject:userData options:0 error:&error];
        
        if (jsonData && !error) {
            NSString *jsonString = [[NSString alloc] initWithData:jsonData encoding:NSUTF8StringEncoding];
            handle_apple_sign_in_callback([jsonString UTF8String], true);
        } else {
            NSDictionary *errorInfo = @{
                @"error": error ? error.localizedDescription : @"Failed to serialize user data"
            };
            NSData *errorJsonData = [NSJSONSerialization dataWithJSONObject:errorInfo options:0 error:nil];
            NSString *errorJsonString = [[NSString alloc] initWithData:errorJsonData encoding:NSUTF8StringEncoding];
            
            handle_apple_sign_in_callback([errorJsonString UTF8String], false);
        }
    }
}

- (void)authorizationController:(ASAuthorizationController *)controller didCompleteWithError:(NSError *)error API_AVAILABLE(ios(13.0)) {
    NSDictionary *errorInfo = @{
        @"error": error.localizedDescription ?: @"Unknown error",
        @"error_code": @(error.code)
    };
    
    NSData *jsonData = [NSJSONSerialization dataWithJSONObject:errorInfo options:0 error:nil];
    NSString *jsonString = [[NSString alloc] initWithData:jsonData encoding:NSUTF8StringEncoding];
    
    handle_apple_sign_in_callback([jsonString UTF8String], false);
}

#pragma mark - ASAuthorizationControllerPresentationContextProviding

- (ASPresentationAnchor)presentationAnchorForAuthorizationController:(ASAuthorizationController *)controller API_AVAILABLE(ios(13.0)) {
    // Get the key window for presenting the authorization UI
    UIWindow *keyWindow = nil;
    
    if (@available(iOS 15.0, *)) {
        NSArray<UIWindowScene *> *windowScenes = [[[UIApplication sharedApplication] connectedScenes] allObjects];
        for (UIWindowScene *windowScene in windowScenes) {
            if (windowScene.activationState == UISceneActivationStateForegroundActive) {
                keyWindow = windowScene.keyWindow;
                break;
            }
        }
    }
    
    // Fallback for older iOS versions or if no key window found
    if (!keyWindow) {
        keyWindow = [[UIApplication sharedApplication] keyWindow];
    }
    
    // Final fallback - get the first window
    if (!keyWindow && [[UIApplication sharedApplication] windows].count > 0) {
        keyWindow = [[[UIApplication sharedApplication] windows] firstObject];
    }
    
    return keyWindow ?: [[UIWindow alloc] init];
}

@end

#pragma mark - C Interface for Rust FFI

void start_apple_sign_in(void) {
    dispatch_async(dispatch_get_main_queue(), ^{
        [[AppleSignInBridge sharedInstance] startAppleSignIn];
    });
}