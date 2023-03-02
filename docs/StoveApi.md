# \StoveApi

All URIs are relative to *https://www.rika-firenet.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_stoves**](StoveApi.md#list_stoves) | **GET** /web/summary | List available stoves
[**login**](StoveApi.md#login) | **POST** /web/login | Login to Rika Firenet API
[**logout**](StoveApi.md#logout) | **GET** /web/logout | Logout from Rika Firenet API
[**stove_controls**](StoveApi.md#stove_controls) | **POST** /api/client/{stoveId}/controls | Set stove parameters
[**stove_status**](StoveApi.md#stove_status) | **GET** /api/client/{stoveId}/status | Get stove status



## list_stoves

> String list_stoves(connect_period_sid)
List available stoves

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connect_period_sid** | **String** | Session cookie | [required] |

### Return type

**String**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/html

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## login

> login(email, password)
Login to Rika Firenet API

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**email** | **String** | Rika Firenet account email | [required] |
**password** | **String** | Rika Firenet password | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## logout

> logout(connect_period_sid)
Logout from Rika Firenet API

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connect_period_sid** | **String** | Session cookie | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## stove_controls

> stove_controls(connect_period_sid, stove_id, room_power_request, bake_temperature, convection_fan1_active, convection_fan1_area, convection_fan1_level, convection_fan2_active, convection_fan2_area, convection_fan2_level, debug0, debug1, debug2, debug3, debug4, eco_mode, frost_protection_active, frost_protection_temperature, heating_power, heating_time_fri1, heating_time_fri2, heating_time_mon1, heating_time_mon2, heating_time_sat1, heating_time_sat2, heating_time_sun1, heating_time_sun2, heating_time_thu1, heating_time_thu2, heating_time_tue1, heating_time_tue2, heating_time_wed1, heating_time_wed2, heating_times_active_for_comfort, on_off, operating_mode, revision, set_back_temperature, target_temperature, temperature_offset)
Set stove parameters

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connect_period_sid** | **String** | Session cookie | [required] |
**stove_id** | **String** | Stove identifier | [required] |
**room_power_request** | Option<**i32**> |  |  |
**bake_temperature** | Option<**String**> |  |  |
**convection_fan1_active** | Option<**bool**> |  |  |
**convection_fan1_area** | Option<**i32**> |  |  |
**convection_fan1_level** | Option<**i32**> |  |  |
**convection_fan2_active** | Option<**bool**> |  |  |
**convection_fan2_area** | Option<**i32**> |  |  |
**convection_fan2_level** | Option<**i32**> |  |  |
**debug0** | Option<**i32**> |  |  |
**debug1** | Option<**i32**> |  |  |
**debug2** | Option<**i32**> |  |  |
**debug3** | Option<**i32**> |  |  |
**debug4** | Option<**i32**> |  |  |
**eco_mode** | Option<**bool**> |  |  |
**frost_protection_active** | Option<**bool**> |  |  |
**frost_protection_temperature** | Option<**String**> |  |  |
**heating_power** | Option<**i32**> |  |  |
**heating_time_fri1** | Option<**String**> |  |  |
**heating_time_fri2** | Option<**String**> |  |  |
**heating_time_mon1** | Option<**String**> |  |  |
**heating_time_mon2** | Option<**String**> |  |  |
**heating_time_sat1** | Option<**String**> |  |  |
**heating_time_sat2** | Option<**String**> |  |  |
**heating_time_sun1** | Option<**String**> |  |  |
**heating_time_sun2** | Option<**String**> |  |  |
**heating_time_thu1** | Option<**String**> |  |  |
**heating_time_thu2** | Option<**String**> |  |  |
**heating_time_tue1** | Option<**String**> |  |  |
**heating_time_tue2** | Option<**String**> |  |  |
**heating_time_wed1** | Option<**String**> |  |  |
**heating_time_wed2** | Option<**String**> |  |  |
**heating_times_active_for_comfort** | Option<**bool**> |  |  |
**on_off** | Option<**bool**> |  |  |
**operating_mode** | Option<**i32**> |  |  |
**revision** | Option<**i32**> |  |  |
**set_back_temperature** | Option<**String**> |  |  |
**target_temperature** | Option<**String**> |  |  |
**temperature_offset** | Option<**String**> |  |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: text/html

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## stove_status

> crate::models::StoveStatus stove_status(connect_period_sid, stove_id)
Get stove status

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connect_period_sid** | **String** | Session cookie | [required] |
**stove_id** | **String** | Stove identifier | [required] |

### Return type

[**crate::models::StoveStatus**](StoveStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

