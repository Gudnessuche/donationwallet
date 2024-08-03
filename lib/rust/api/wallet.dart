// This file is automatically generated, so please do not edit it.
// Generated by `flutter_rust_bridge`@ 2.0.0-dev.37.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'structs.dart';

Future<String> setup(
        {required String label,
        String? mnemonic,
        String? scanKey,
        String? spendKey,
        required int birthday,
        required String network}) =>
    RustLib.instance.api.crateApiWalletSetup(
        label: label,
        mnemonic: mnemonic,
        scanKey: scanKey,
        spendKey: spendKey,
        birthday: birthday,
        network: network);

/// Change wallet birthday
/// Reset the output list and last_scan
String changeBirthday({required String encodedWallet, required int birthday}) =>
    RustLib.instance.api.crateApiWalletChangeBirthday(
        encodedWallet: encodedWallet, birthday: birthday);

/// Reset the last_scan of the wallet to its birthday, removing all outpoints
String resetWallet({required String encodedWallet}) => RustLib.instance.api
    .crateApiWalletResetWallet(encodedWallet: encodedWallet);

Future<void> syncBlockchain({required String network}) =>
    RustLib.instance.api.crateApiWalletSyncBlockchain(network: network);

Future<String> scanToTip(
        {required String encodedWallet, required String network}) =>
    RustLib.instance.api.crateApiWalletScanToTip(
        encodedWallet: encodedWallet, network: network);

WalletStatus getWalletInfo({required String encodedWallet}) =>
    RustLib.instance.api
        .crateApiWalletGetWalletInfo(encodedWallet: encodedWallet);

String markOutpointsSpent(
        {required String encodedWallet,
        required String spentBy,
        required List<String> spent}) =>
    RustLib.instance.api.crateApiWalletMarkOutpointsSpent(
        encodedWallet: encodedWallet, spentBy: spentBy, spent: spent);

String addOutgoingTxToHistory(
        {required String encodedWallet,
        required String txid,
        required List<String> spentOutpoints,
        required List<Recipient> recipients}) =>
    RustLib.instance.api.crateApiWalletAddOutgoingTxToHistory(
        encodedWallet: encodedWallet,
        txid: txid,
        spentOutpoints: spentOutpoints,
        recipients: recipients);

String addIncomingTxToHistory(
        {required String encodedWallet,
        required String txid,
        required Amount amount,
        required int height}) =>
    RustLib.instance.api.crateApiWalletAddIncomingTxToHistory(
        encodedWallet: encodedWallet,
        txid: txid,
        amount: amount,
        height: height);

String? showMnemonic({required String encodedWallet}) => RustLib.instance.api
    .crateApiWalletShowMnemonic(encodedWallet: encodedWallet);
